"""
Setup script for the magics_gym package.
"""

from setuptools import setup, find_packages

setup(
    name="magics_gym",
    version="0.1.0",
    description="OpenAI Gym environment for the Magics simulation",
    author="Magics Team",
    packages=find_packages(),
    install_requires=[
        "gym>=0.21.0",
        "numpy>=1.20.0",
    ],
    python_requires=">=3.8",
)
