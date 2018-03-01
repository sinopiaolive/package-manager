## Files

The `files` field, controlling which files to release.

```
files: {
  include: possible values:
    [ "**/*.js", ... ] // explicit list of globs
    null
      // auto-detect VCS (default), applying .gitignore
      // throws error if there are uncommitted files
  exclude: defaults to []
    // e.g. [ '/tests' ]
}
```


## Platforms

* Don't deal with platforms at all.
* If some package depends on `foo` for Linux and `bar` for Windows, we just
  download both every time.
* It's the build tool's responsibility to pick up the right packages.


## Private and 3rd-party registries

Project manifest (not for published libraries):

```toml
registries = [
  "private-registry.acme.corp"
]
default_registry = "crates.io" # default, always has lowest priority

[dependencies]
# left-pad does not exist on the private registry, so it comes from the
# default registry.
"rust/left-pad" = "^1.2.3"

# acme-lib exists on the private registry. If anybody were to push acme-lib
# to the public registry, the public acme-lib will be ignored completely.
"rust/acme-lib" = "^1.2.3"

# Somebody chose to push rust/serde to the private registry. Oops!
# This suddenly starts shadowing the publicly-released serde
# package on every project that uses the private registry.
# The solution is: don't do that.
"rust/serde" = "^1.2.3"
```

* Don't put `https://` protocol into registry_url -- we don't want duplicate
  versions of the same package due to different protocols.

* `manifest.lock` records the registry URL for every package at time of
  resolution.

* Private packages like `rust/acme-lib` depend on other private packages (`rust/acme-foo = ^1.2.3`) without mentioning the registry. They rely on the private registry being made available through the consuming project's manifest.

* The main alternative to this approach is making the registry URL explicit for
  each package:

    ```toml
    "rust/acme-lib" = { version = "^1.2.3", registry_url = "internal.acme.corp" }
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


## License fields

* Try to auto-detect license_file if absent.
* Require license tag and/or license_file, to prevent people from accidentally
  publishing *all rights reserved* code.
* If license tag and license file disagree there's a problem, but this doesn't
  seem to happen a lot. It's not our problem.
* We're already nudging users to be explicit on the client, so on the server we
  can now guarantee that we have either tag or file.
* On private registries, we might just allow not having a license. Or we might
  let the server decide.


## Manifest syntax

```
registries [
  "internal.google.com"
]

dependencies {
  js/left-pad ^2.0.0
  js/right-pad ^2.0.0
  js/tokio ^2.0.0

  js/mocha ^1.2.3 dev
  js/debugger ^1.2.3 dev

  js/foo git="https://github.com/joliss/foo"
  js/bar path="C:\\Program Files\\bar"
  js/up-pad ^2.0.0
}

package {
  name "js/mypkg"
  version "1.0.0"

  authors [ "Bodil Stokke <bodil@bodil.org>" ]
  description "The description."
  license_file "license/GPL"
  license "MIT"
  keywords []

  files {
    // Add all files tracked by Git:
    add_committed
    // Alternatively, add only some of the files tracked by Git:
    //add_committed "src/"
    //add_committed "data/*.json"

    // Add some files not in Git:
    //add "src/generated/**/*.rs"

    // But do not include the following:
    //remove "vendor/"
  }

  registry "internal.google.com"
}
```

## Things to implement before releasing v1

We want to get a usable and demo-able pre-1.0 version to solicit feedback -- it
needs to be usable mainly as a vendoring tool -- then implement the remaining
things for 1.0, after which we want to keep breakage to a minimum.

* [ ] Enforce file name and path length limitations ([Naming Files, Paths, and Namespaces (Windows)](https://msdn.microsoft.com/en-us/library/windows/desktop/aa365247(v=vs.85).aspx))

* [ ] Package dependency on PM version (supplied by registry, not by package).

* [ ] Virtual dependencies / engines

    * 1 version (compiler)
    * n versions (browser)
    * the Package Manager itself?

## Things to plan for before releasing v1

For features that we don't want to implement in v1, we may still want to get a
general idea of whether and how we plan on supporting them later, just to make
sure we don't back ourselves into a corner. These features include:

* [x] Private registries

* [x] Platforms

* [ ] Cargo [features](http://doc.crates.io/manifest.html#the-features-section)

* [ ] Nested npm-style dependencies

* [ ] Git URLs

* [ ] Local paths

    * for local development

    * for workspaces

* [ ] Overriding dependencies (e.g. [Cargo's `[patch]` and `[replace]`](http://doc.crates.io/specifying-dependencies.html#overriding-dependencies))

* [ ] Namespaces like npm's
  `@babel/everything-here-is-reserved-for-the-babel-project`

* [ ] Jo wants to try Nix (it's user-local and really cool);
  also Guix is a Nix clone with a better-documented API language

* [ ] That compatibility thing Jo is writing a blog post about
