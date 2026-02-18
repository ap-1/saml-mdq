# ContactPerson deserialization bug in samael

When fetching SAML metadata from [InCommon MDQ](https://mdq.incommon.org) (e.g., for `https://login.cmu.edu/idp/shibboleth`), samael fails to deserialize `<ContactPerson>` elements with:

```
duplicate field `@contactType`
```

This happens because real-world SAML metadata from federations like InCommon includes both standard and namespaced attributes on `<ContactPerson>`:

```xml
<ContactPerson contactType="technical" remd:contactType="http://refeds.org/metadata/contactType/security">
```

The underlying XML library (quick-xml) strips the `remd:` prefix during serde deserialization, so both `contactType` and `remd:contactType` resolve to the same serde field name `@contactType`, triggering a duplicate field error.

A secondary issue was that `contact_person` on `EntityDescriptor` was typed as `Option<Vec<ContactPerson>>` without `#[serde(default)]`, which also caused problems when multiple `<ContactPerson>` sibling elements appeared in metadata. This is the same class of bug originally reported in [samael#11](https://github.com/njaremko/samael/issues/11), which was closed in 2022 by [PR #17](https://github.com/njaremko/samael/pull/17) fixing the `X509Certificate` duplicate field case but not the `contactType` one.

## Bandaid patch 

The fix lives on the [`fix/contact-person-deserialization`](https://github.com/ap-1/samael/tree/fix/contact-person-deserialization) branch of a [samael fork](https://github.com/ap-1/samael). It's a single commit ([`953e5aa`](https://github.com/ap-1/samael/commit/953e5aa4)) that changes three files:

- `src/metadata/contact_person.rs` -- replaces the derive-based `Deserialize` impl on `ContactPerson` with a hand-written `impl<'de> serde::Deserialize<'de> for ContactPerson`. The custom visitor handles duplicate keys (e.g., `@contactType` appearing twice due to `remd:contactType` being stripped to `contactType`) by keeping the first value and skipping the duplicate instead of erroring. It also accumulates repeated `EmailAddress`/`TelephoneNumber` entries into their `Vec`s.

- `src/metadata/entity_descriptor.rs` -- changes `contact_person` from `Option<Vec<ContactPerson>>` to `Vec<ContactPerson>` with `#[serde(default)]`, matching the pattern already used by `RoleDescriptor`, `IdpSsoDescriptor`, and other structs in samael. Adds tests for both multiple `<ContactPerson>` elements and the namespaced attribute case.

- `src/service_provider/mod.rs` -- adjusts the `ServiceProvider` builder to use `.unwrap_or_default()` since the field is now a plain `Vec`.

This fork is currently pinned in `Cargo.toml`:

```toml
samael = { git = "https://github.com/ap-1/samael.git", branch = "fix/contact-person-deserialization", features = ["xmlsec"] }
```

This is being upstreamed via [njaremko/samael#80](https://github.com/njaremko/samael/pull/80).

## Why quick-xml strips namespace prefixes

This is a deliberate pragmatic choice, introduced by Mingun in [PR #490](https://github.com/tafia/quick-xml/pull/490) (Oct 2022) as part of a major serde rewrite. The reasoning, stated explicitly in [issue #757](https://github.com/tafia/quick-xml/issues/757):

> "namespace prefixes does not considered as part of field name because there can be different prefixes for the one namespace even in the same document."

Since the same namespace URI can be bound to different prefixes in different parts of a document (e.g., `xmlns:foo="urn:example"` in one element, `xmlns:bar="urn:example"` in another), matching on prefixes in serde struct field names would break whenever the producer chose a different prefix. Local names are the "stable" part, so `local_name()` was used as a simplification.

However, this is wrong per the [W3C Namespaces in XML spec (Section 6.2)](https://www.w3.org/TR/xml-names/#defaulting): unprefixed attributes are not in any namespace, prefixed attributes are in the namespace bound to that prefix, and they are distinct attributes -- `<good a="1" n1:a="2" />` is legal because `a` (no namespace) and `n1:a` (in `n1`'s namespace) have different expanded names. Stripping the prefix makes two spec-distinct attributes appear identical, which is exactly the `contactType` vs `remd:contactType` problem.

This is tracked across several open quick-xml issues:

- [#218 "Struct namespaces with Serde"](https://github.com/tafia/quick-xml/issues/218) -- master tracking issue, open since 2020
- [#347 "Inconsistent namespace stripping with serde"](https://github.com/tafia/quick-xml/issues/347) -- labeled `bug`, still open
- [#757 "duplicate field `@type`"](https://github.com/tafia/quick-xml/issues/757) -- the exact same class of bug with `type` vs `xsi:type`

Partial fixes have been merged incrementally: `xmlns:` prefixes preserved in [PR #539](https://github.com/tafia/quick-xml/pull/539), `xml:` prefixes preserved in [PR #873](https://github.com/tafia/quick-xml/pull/873). But all other namespace prefixes are still stripped.

A proper fix requires namespace-aware serde, which is architecturally hard since serde's data model has no concept of XML namespaces. There's an active WIP by jespersm using James Clark notation -- encoding full namespace URIs into serde rename strings like `@{urn:example}contactType` -- but it's not ready for PR yet. The maintainers have explicitly welcomed contributions. An interim fix preserving all prefixes (following the pattern of PRs [#539](https://github.com/tafia/quick-xml/pull/539) and [#873](https://github.com/tafia/quick-xml/pull/873)) would likely be accepted, though there's tension with users who rely on prefix stripping to handle varying prefixes across documents.

## Next steps

- ~~Open a PR on [njaremko/samael](https://github.com/njaremko/samael) upstreaming the fork's fix, referencing [samael#11](https://github.com/njaremko/samael/issues/11)~~ Done: [njaremko/samael#80](https://github.com/njaremko/samael/pull/80)
- Comment on [quick-xml #218](https://github.com/tafia/quick-xml/issues/218) or [#757](https://github.com/tafia/quick-xml/issues/757) linking this use case as another example of real-world breakage
- Attempt an interim PR on quick-xml to preserve all prefixes (not just `xmlns:` and `xml:`), following the pattern of PRs [#539](https://github.com/tafia/quick-xml/pull/539)/[#873](https://github.com/tafia/quick-xml/pull/873)
- Remove the fork pin in `Cargo.toml` once the fix is merged upstream
