## JSON Schema tools

[![Rust build status](https://img.shields.io/github/actions/workflow/status/travisbrown/json-schema-tools/ci.yaml?branch=main)](https://github.com/travisbrown/json-schema-tools/actions)

This project includes a small Rust library and command-line tool that do two things with [JSON Schema][json-schemas] documents:

* Lint them.
* Combine a collection of schemas linked via schema references into a single schema suitable for use with e.g. [Typify][typify].

In both cases only a small subset of schemas are supported (the ones I need for my own use cases).

# License
This software is published under the [GNU General Public License v3.0][gplv3].

[gplv3]: https://www.gnu.org/licenses/gpl-3.0.en.html
[json-schemas]: https://json-schema.org/
[typify]: https://github.com/oxidecomputer/typify
