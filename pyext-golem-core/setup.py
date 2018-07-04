from setuptools import setup
from setuptools_rust import Binding, RustExtension

setup(
    name='golem-core',
    version='0.1.0',
    url='https://golem.network',

    packages=['golem_core'],
    rust_extensions=[RustExtension(
        'libgolem_core',
        'Cargo.toml',
        binding=Binding.PyO3,
        debug=False,
    )],

    zip_safe=False
)
