# envy store [![Build Status](https://travis-ci.org/softprops/envy-store.svg?branch=master)](https://travis-ci.org/softprops/envy-store) [![Software License](https://img.shields.io/badge/license-MIT-brightgreen.svg)](LICENSE) [![crates.io](http://meritbadge.herokuapp.com/envy-store)](https://crates.io/crates/envy-store) [![Released API docs](https://docs.rs/envy-store/badge.svg)](http://docs.rs/envy-store) [![Master API docs](https://img.shields.io/badge/docs-master-green.svg)](https://softprops.github.io/envy-store)

> 🏪 deserialize [AWS Parameter Store](https://docs.aws.amazon.com/systems-manager/latest/userguide/systems-manager-paramstore.html) values into typesafe structs

> 👭 Consider this a cousin of [envy](https://github.com/softprops/envy) a crate for deserializing environment variables into typesafe structs.

## 📦 Install

```toml
[dependencies]
envy-store = "0.1"
```

## 🤸 Usage

See the [demo example](examples/demo.rs) for an example application and [documentation](https://softprops.github.io/envy-store) for more information

## 🤔 Why AWS Parameter Store

Environment variables are a perfectly good and probably
best solution for storing application configuration as they are more or less
universally supported across runtimes and languages.

As an application grows additional factors need may come into consideration.

1) Security. Environment variables alone are a poor transport for secret information
as they can easily be leaked in their plain text format. AWS Parameter Store has
built-in support for storing values in encrypted format preventing unwanted access
from prying eyes.

2) Management. The strategy for configuring environment variables for your application
will likely vary and become less managable over time. The source of truth for their
values may require some centralization in order to manage. Systems exist for helping
you manage these. AWS Parameter Store is a self managed system as a service removing
the need for you to operate one of these systems yourself.

3) Access control. Related to encryption security, you may also want to limit _who_ can access
configuration. Identity access management is built into AWS so you don't have
to implement this yourself.

Doug Tangren (softprops) 2018