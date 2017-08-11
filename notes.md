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

Project manifest (not for published libraries):

```toml
registries = [
  'private-registry.acme.corp'
]
default_registry = 'crates.io' # default, always has lowest priority

[dependencies]
# left-pad does not exist on the private registry, so it comes from the
# default registry.
rust/left-pad = '^1.2.3'

# acme-lib exists on the private registry. If anybody were to push acme-lib
# to the public registry, the public acme-lib will be ignored completely.
rust/acme-lib = '^1.2.3'

# Somebody chose to push rust/serde to the private registry. Oops!
# This suddenly starts shadowing the publicly-released serde
# package on every project that uses the private registry.
# The solution is: don't do that.
rust/serde = '^1.2.3'
```

* Don't put `https://` protocol into registry_url -- we don't want duplicate
  versions of the same package due to different protocols.

* `manifest.lock` records the registry URL for every package at time of
  resolution.

* Private packages like `rust/acme-lib` depend on other private packages (`rust/acme-foo = ^1.2.3`) without mentioning the registry. They rely on the private registry being made available through the consuming project's manifest.

* The main alternative to this approach is making the registry URL explicit for
  each package:

    ```toml
    rust/acme-lib = { version: '^1.2.3', registry_url: 'internal.acme.corp' }
    ```

  The main disadvantage here is that we need all packages to agree on where the
  `rust/acme-lib` dependency comes from, which likely means that the solver
  needs to be aware of the registry URL, either as part of the `PackageName` or
  as part of the version:

    ```rust
    struct Versionoid {
      version: Version,
      registry_url: Option<String>
    }
    ```

  Either approach comes with problems (in implementation complexity and
  sometimes in how it interacts with git URLs), and we couldn't find a solution
  that we found satisfactory.
