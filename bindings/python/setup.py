
import sys
import platform

from setuptools import setup
from setuptools_rust import RustExtension


def get_py_version_cfgs():
    # For now each Cfg Py_3_X flag is interpreted as "at least 3.X"
    version = sys.version_info[0:3]
    py3_min = 9
    out_cfg = []
    for minor in range(py3_min, version[1] + 1):
        out_cfg.append("--cfg=Py_3_%d" % minor)

    if platform.python_implementation() == "PyPy":
        out_cfg.append("--cfg=PyPy")

    return out_cfg


setup(
    name="iota_sdk",
    version="1.1.2",
    classifiers=[
        "License :: SPDX-License-Identifier ::  Apache-2.0",
        "Intended Audience :: Developers",
        "Programming Language :: Python",
        "Programming Language :: Rust",
        "Operating System :: POSIX",
        "Operating System :: MacOS :: MacOS X",
    ],
    packages=["iota_sdk"],
    rust_extensions=[
        RustExtension(
            "iota_sdk.iota_sdk",
            rustc_flags=get_py_version_cfgs(),
            debug=False,
        ),
    ],
    include_package_data=True,
    zip_safe=False,
    install_requires=["dacite >= 1.8.1 ; pyhumps >= 3.8.0"],
    package_data={"iota_sdk": ["py.typed"]}
)
