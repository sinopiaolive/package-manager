# A Cross-Environment Package Manager

[![Build Status](https://travis-ci.org/sinopiaolive/package-manager.svg?branch=master)](https://travis-ci.org/sinopiaolive/package-manager)

This as-yet unnamed Package Manager resolves and fetches project-local
dependencies in an environment-agnostic (language-agnostic) way.

Think of it as a Bundler that is language-independent. Alternatively, think of
it as a Bower with dependency resolution that resolves conflicts automatically,
rather than requiring user intervention.

The Package Manager has its own registry. It's not a drop-in replacement for any
existing package manager.

## Usage

This is work-in-progress code, and you cannot use it to do anything yet.

## Motivation

At the moment, every language environment (such as Ruby or Rust) has its own
package manager (such as Bundler). All those package managers tend to roughly
solve the same problem with only slight variations between them. This Package
Manager is an attempt to solve the problem once and for all.

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
practice. To the degree that it is not, we may then generalize the code, making
choices such as the format for version numbers parametrizable, so that it can be
used as a framework to produce custom package managers.

### Why not use npm?

It is possible to use npm to distribute code other than JavaScript. However,
npm's main shortcoming for our purposes is that it exclusively uses *nested
dependencies*. We want *flat dependencies* as our primary use case. (TODO: Link
a document that explains what the difference is, why flat dependencies matter to
library authors, and why npm's peer dependencies and deduping don't fully solve
the problem for us.)

### Why not use Bower?

Bower's flat dependency resolution isn't fully automatic: When it encounters a
conflict, it asks the user what to do. We want to run without intervention,
producing either a set of package versions or an error saying "it's impossible
to satisfy the dependencies you specified, and here's why" -- just like Bundler.

## What's included?

* Server functionality to host a registry of packages.

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

## Dependency resolution

Given a set of dependencies, we perform *flat dependency resolution*.

Formally, we obtain a solution S mapping Package Names to Version numbers, where
each of the initial dependencies and the dependencies of each package contained
in S are satisfied by S. S is minimal: no package can be removed from S without
breaking a dependency.

Our solver performs an exhaustive (but not brute force) search. If there exists
a solution, it will eventually find it, and if there doesn't exist one, it will
eventually return an error indicating a conflict. While a brute force search
would have exponential complexity, we find that with a simple inference
optimization, the solver handles all dependency sets we've tested so far in
milliseconds. If we run into cases that the solver doesn't handle well, there is
potential for more optimizations.

The solver could conceivably be made generic over the Package Name, Version, and
Version Constraint types and extracted into a separate library -- like
[Molinillo](https://github.com/CocoaPods/Molinillo) but with a Rust interface.

### Supporting npm-style dependencies

It is possible to add support for packages to additionally have npm-style
"private" dependencies that are not shared with other packages. We don't know of
a package manager that currently allows mixing flat and nested resolution, but
it might turn out to be quite useful in practice.

One open question is *who* should specify that a dependency is private: the
depender ("I only want a private copy of the following package") or the dependee
("this package is always a private dependency -- you can have duplicates of
it").

## Design

Package managers differ in subtle ways. While implementing the Package Manager,
we're having to make a lot of choices. This section documents those choices.

The goal of this section is twofold:

1. By documenting precisely what we're (planning on) doing, we can solicit
   feedback and check if those choices are suitable for existing languages and
   projects. For example, in the Haskell ecosystem, many libraries use four
   version components (like
   [1.2.6.1](https://hackage.haskell.org/package/hashable)), so if we forced all
   version numbers to conform to a major.minor.patch format as prescribed by
   semver, those libraries couldn't be uploaded to the Package Manager's
   registry.

2. If we want to turn the Package Manager into a framework for writing custom
   package managers, this section serves as a list of things that could be made
   generic. For example, we might want to be generic over the Version type to
   allow for custom version number formats, and merely provide our Version
   implemention as a suggested default.

<!-- Note: For ease of exposition, the terminology in this section does not
always match our struct names precisely. -->

### Registry data

The Registry stores Package Names. Each Package Name has ownership information
and a list of Versions. Each Version has Package Metadata, an Archive containing
the package contents, and a Dependency Set.

### Dependency Set

A Dependency Set maps Package Names to Version Constraints.

### Package Names

Package Names are of the format `<namespace>/<name>`, where `<namespace>` and
`<name>` are case sensitive strings chosen freely by the package author, with
`<namespace>` matching `[a-z0-9_][a-z0-9_-]*` (up to 128 characters) and `<name>`
matching `[a-zA-Z0-9_][a-zA-Z0-9_-]*` (up to 128 characters).

Conflicting package name capitalizations are disallowed by the registry.

### Version numbers

We use the [semver.org](http://semver.org/) standard for our Version format,
with the following changes:

* Instead of requiring three number fields (major.minor.patch), a Version may
  have one or more number fields for the base version. Trailing zeros in the
  base version are ignored for the purpose of equality and comparison, e.g.
  1.2-beta == 1.2.0.0-beta, but 1.2-beta != 1.2-beta.0.0. However, we still
  preserve them for printing.

* The number fields and numeric pre-release fields must fit `u64`.

* Version numbers can be up to 128 characters long.

* We do not allow build metadata (semver section 10), e.g. 1.0.0+sha.5114f85,
  because it does not appear to be used widely
  ([thread](https://twitter.com/jo_liss/status/879671042989580288)). *[The
  alternative is to allow it but ignore it like trailing zeros.]*

Conflicting version capitalizations are disallowed by the registry.

#### Version priority

Versions are ordered as defined in semver section 11.

We typically want the solver to pick the highest possible version for each
package. However, we need to deal with pre-release versions: for example, if we
depend on version `^1.0.0` and a package has versions 1.0.0, 1.1.0-beta, 1.1.0,
and 1.2.0-beta, then the "best" version is 1.1.0. Even though 1.2.0-beta is a
higher version, we only want to pick it if some other package's dependencies
preclude us from picking 1.1.0.

To achieve this, we additionally define a secondary "priority" ordering by
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

The Version Constraint type is used to implement expressions like `>= 1.2.0`. It
serves as a predicate for Versions by implementing
`VersionConstraint::matches(Version) -> bool`.

We define the following format for Version Constraints:

* `<version>`: matches only the exact Version (up to trailing zeros)

* `*`: matches any Version

* `>= <version>`: matches any Version greater-equal `version` as per semver
  ordering

* `< <version>`: matches any Version less than `version` as per semver ordering

* `>= <ver1> < <ver2>`, where `<ver2>` must be greater than `<ver1>`: matches
  any Version that matches both `>= <ver1>` and `< <ver2>`

  Observe that we do not expect `>= 1.0 < 2.0` to match 2.0-beta.1 even though
  2.0-beta.1 orders before 2.0. To achieve this, we define the following
  exception for constraints of the form `>= <ver1> < <ver2>` and `< <ver2>`:

  If `ver2` has no pre-release tags, then this does not match
  `<ver2>-<any.pre.release.tag>`, unless `ver1` is given and its base version
  equals `ver2` (up to trailing zeros). For example, the following constraints
  do not match version 2.0-beta.1:

    * `>= 1.0 < 2.0`

    * `< 2.0`

  However, the following constraints do match version 2.0-beta.1:

    * `>= 1.0 < 2.1`

    * `>= 1.0 < 2.0-beta.2`

    * `>= 2.0-beta.1 < 2.0`

* `^<version>`: matches any version that is `>= <version>` and starts with the
  same digit. For example, `^1.2` matches any `1.x` version that is `>= 1.2`.

  If `version` has leading zeros, the first non-zero digit must match. For
  example, `^0.0.1.2` matches any `^0.0.1.x` version that is `>= 0.0.1.2`.

  `version` must be greater than `0`.

We allow zero or more spaces after `>=`, `<`, and `^`. However, when printing
version constraints, we use the canonical amount of whitespace as written above.

[TODO: Discuss the possibility of allowing optional constraints and negative
constraints (exclusions).]

### Package Metadata

This metadata is used to produce registry web pages, print errors, and
facilitate searching.

* `description: String`

* `license: Option<String>`

  *[Should we force this to conform to some set of license codes?]*

* `license_file: Option<String>`

  *[Should we require that this file exists, and that the file path is in
  canonical form with forward slashes?]*

* `homepage: Option<String>`

* `bugs: Option<String>`

  This is apparently useful for directing people to the appropriate bug tracker
  if a tool needs to print an error message because something went wrong with a
  package. *[Is it really necessary?]*

* `repository: Option<String>`

  *[Make this a structure along the lines of `{ type: String, url: String }`?
  Should we optionally also automatically store the commit that produced this
  release?]*

* `keywords: Vec<String>`

*[Do we want to restrict the set of Unicode scalars that are allowed in these
strings?]*

### Archives

TODO

### Package ownership and user accounts

Tbd.

## Other questions to explore

* Separate inward-facing (your project consumes other libraries) and
  outward-facing (your project is published for consumption by other libraries)
  manifests? Apps only use the inward-facing part, libraries use both.

* Can we make it possible for packages to provide "compatibility shims" for old
  versions of themselves, so that v2.0.0 can also pretend to be v1.0.0?
