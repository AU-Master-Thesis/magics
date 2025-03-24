from setuptools import setup, find_packages

setup(
    name="magics_gym",
    version="0.1.0",
    description="OpenAI Gym environment for the Magics simulation",
    author="Magics Team",
    packages=find_packages(),
    install_requires=[
        "numpy>=2.2.4",
        "pyzmq>=22.0.0",
        "gymnasium>=0.26.0",
    ],
    python_requires=">=3.10",
)
