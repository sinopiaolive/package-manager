```
struct Manifest:
  pm : String // package manager version
  name : String
  version : String
  private/publish : Boolean
  dependencies : Array<VersionedPackageSpec>
    name : String
    constraint : VersionConstraint
      // version must be >= minVersionOrNull, < maxVersionOrNull
      ( minVersionOrNull, maxVersionOrNull )
  devDependencies : (same)

  files:
    include: possible values:
      [ "**/*.js", ... ] // explicit list of globs
      null
        // auto-detect VCS (default), applying .gitignore
        // throws error if there are uncommitted files
    exclude: defaults to []
      // e.g. [ '/tests' ]

  // Not crucial but useful metadata:
  author : String
  description : String
  license : String
  license-file: String // default: whatever we find in [ 'LICENSE', 'LICENSE.*', ... ]
  homepage, documentation, bugs : String
  repository : Repository
  keywords : Array<String>

union VersionConstraint:
  Version
  Pair(Option<Version>, Option<Version>)

struct Version:
  // start with Steve's semver library, only allowing a.b.c
  // later we can maybe allow arbitrary number a.b.c.d.e
```

Later: Dependencies for platform:
  - hard to agree on what's a platform (unix, windows, mac? ruby, jruby? node, browser?)
  - have arbitrary set of platform tags, allow "dependencies.(!jruby && mac)"
  - print warning if no build-tool-supplied platform

Later: privateDependencies/staticDependencies

Later: externalDependencies, e.g. rustc: ~1.1.2; like engine

Later: features?

Later: npm link scenarios

- workspaces: groups of related packages developed together

- git dependencies: committing interdependent work-in-progress for other people to test

- replace: pulling hotfixes into an app

    - redundant with git dependencies?

- .cargo/config: testing out fixes locally


## Private and 3rd-party registries

```
primary_registry = 'crates.io' // default

// `namespace` comes from primary registry, `@namespace` comes from secondary
// registries:
[registries]
// @mozilla_nightly namespace comes from this registry & namespace
mozilla_nightly = 'nightly.mozilla.org#mozilla_nightly'
// @anything_else namespaces come from this registry
* = 'https://private-pmreg.acme.com'

[dependencies]
left-pad = '1.2.3'
@rust/our_private_library = '1.2.3' // from private-pmreg.acme.com
@mozilla_nightly/servo = '1.2.3'
// This is sugar for this absolute package syntax:
nightly.mozilla.org/pm#mozilla_nightly/servo = '1.2.3'
```

```
PackageName {
    registry_url: Option<String>,
    namespace: String,
    name: String
}
```

Don't put `https://` protocol into registry_url -- we don't want duplicate
versions of the same package due to different protocols.

Installed packages: `pm info --json`

```
[
    {
        registry_url: "nightly.mozilla.org",
        namespace: "rust",
        name: "servo",
        version: "1.2.3",
        path: ".../fasdlkfjsdlafkj/servo"
    },
    ...
]
```
