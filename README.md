# A Cross-Environment Package Manager

[![Build Status](https://travis-ci.org/sinopiaolive/package-manager.svg?branch=master)](https://travis-ci.org/sinopiaolive/package-manager)

This as-yet unnamed Package Manager resolves and fetches project-local
dependencies in an environment-agnostic (language-agnostic) way.

## Usage

This is work-in-progress code, and you cannot use it to do anything yet.

## Motivation

At the moment, every language environment (such as Ruby or Rust) has its own
package manager (such as Bundler or Cargo). All those package managers tend to
roughly solve the same problem with only slight variations between them. This
Package Manager is an attempt to solve the problem once and for all.

It does not aim to unify all package managers or serve as a drop-in replacement
for any existing package manager. Rather, it aims to obviate the need to write a
new package manager for every new language, and perhaps to serve as a package
manager for existing language ecosystems that don't currently have a
satisfactory package manager of their own.

### Is this a stand-alone project?

It is not yet clear to us how much integration is needed between a package
manager and a language's build tools, and how many genuine differences there are
between languages in what they require their package managers to do. In other
words, we're asking the following open question:

Is it possible to share between multiple languages a single package management
command line tool and a single registry (with packages namespaced by language)
without sacrificing usability?

* If yes, then the Package Manager will be primarily a stand-alone tool, used
  from the command line or via API calls from other tools.

* If not, then the Package Manager will be primarily a framework used to write
  customized language-specific package managers and host custom registries with
  a minimum amount of effort. In other words, there will still be a separate
  package manager for every new language, but this Package Manager will contain
  the logic that is shared between them.

Our current approach is to produce a stand-alone tool and see if it is usable in
practice. To the degree that it is not, we may then make the code more generic
(parametrizable) so that it can be used to produce custom package managers.

### Why not use npm?

It is possible to use npm to distribute code other than JavaScript. However,
npm's main shortcoming for our purposes is that it exclusively uses *nested
dependencies*. We want *flat dependencies* as our primary use case.

### Why not use Bower?

Bower's flat dependency resolution isn't fully automatic: When it encounters a
conflict, it asks the user what to do. We want to run without intervention,
producing either a set of package versions or an error saying "it's impossible
to satisfy the dependencies you specified, and here's why" -- just like Bundler
or Cargo.

## What's included?

* Server functionality to host an index of packages and package contents.

* Client functionality for app authors to

    * parse an app manifest,
    * download the registry index,
    * perform flat dependency resolution,
    * store the resolved dependencies in a lock file, and
    * download and unpack the dependencies.

* Client functionality for library publishers to

    * parse a library manifest,
    * authenticate to the registry server, and
    * upload a new library release to the registry server.

## Design

Package managers differ in subtle ways. We're currently working towards a
stand-alone tool. While doing this, we're having to make a lot of choices. This
section documents those choices.

None of these choices are set in stone just yet. If you see a problem with one
of them, please tell us!

If we later turn the Package Manager into a framework, we might abstract over
these questions and only provide our choices as defaults.

### Registry data

The Registry stores Package Names. Each Package Name has ownership information
and a list of Versions. Each Version has Package Metadata, a Tarball, and
dependencies, which is a mapping from Package Names to Version Constraints.

### Package Names

TODO

### Version numbers

We use the [semver.org](http://semver.org/) standard for our version format,
with the following modifications:

* Instead of three numeric fields (major.minor.patch), a Version may have one or
  more fields. Trailing zeros are ignored for the purpose of equality and
  comparison, e.g. 1.2 == 1.2.0.0. This is to allow packages with existing
  versioning schemes that differ from semver
  ([example](https://hackage.haskell.org/package/hashable)) to be uploaded more
  easily.

* We do not allow build metadata (semver section 10), e.g. 1.0.0+sha.5114f85.
  (Alternatively, if we change our minds on this, we will ignore it like
  trailing zeros.)

#### Version priority

Versions are ordered as defined in semver section 11.

We typically want the solver to pick the highest possible version for each
package. However, we need to deal with pre-release versions: For example, if we
depend on version `^1.0.0` and a package has versions 1.0.0, 1.1.0-beta, 1.1.0,
and 1.2.0-beta, then the "best" version is 1.1.0. Even though 1.2.0-beta is a
higher version, we only want to pick it if some other package's dependencies
precludes us from picking 1.1.0.

To achieve this, we additional define a secondary "priority" ordering by
comparing `(version.has_no_prerelease_tag(), version)` tuples, with `false <
true`.

Compare the two orderings for our example:

```
semver (low to high)   priority (worst to best)
===============================================
1.0.0                  1.1.0-beta
1.1.0-beta             1.2.0-beta
1.1.0                  1.0.0
1.2.0-beta             1.1.0
```

The solver uses this priority ordering to determine which version to pick when
there are multiple solutions.

### Version Constraints

TODO

### Package Metadata

TODO

### Tarballs

TODO

### Package ownership and user accounts

Tbd.
