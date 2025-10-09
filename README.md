# Banquo Python Bindings

This repository contains the python bindings for the Banquo temporal logic
monitor, which is written in Rust.

## Installation

We try to provide pre-built wheels for all supported python verions across a
variety of common platforms. These wheels can be installed from
[PyPI](https://pypi.org/package/pybanquo), the central python package registry,
using the command:

```shell
$ pip install pybanquo
```

This project can also be installed directly from GitHub using the command

```shell
$ pip install "pybanquo @ git+https://github.com/cpslab-asu/banquo-python"
```

## Usage

Creating formulas is accomplished by combining logical expressions like
`Predicate` with operators like `And` and `Eventually`. The result of combining
these terms is a temporal logic formula, which can be used to evaluate a
`Trace`, which is a set of system states along with the associated time for
each state. This is an example of this process:

```python

import banquo as bq
import banquo.operators as ops

# x <= 5.0
p1 = bq.Predicate({"x": 1.0}, 5.0)

# -y <= -3.0 => y > 3.0
p2 = bq.Predicate({"y": -1.0}, -3.0)

# always (p1 or p2)
f = ops.Always(ops.Or(p1, p2))

t = bq.Trace({
    0.0: {"x": 4.8, "y": 6.3},
    1.0: {"x": 4.7, "y": 9.3},
    2.0: {"x": 4.9, "y": 5.4},
    3.0: {"x": 4.6, "y": 4.1},
    4.0: {"x": 4.2, "y": 3.1},
})

rho = bq.evaluate(f, t)
```

More advanced usage of the operators, including the bounded `Always` and
`Eventually` operators can be seen in the `tests/test_conformance.py` module.
