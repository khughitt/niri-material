"""Tooling test fixtures and explicit subprocess environments."""
import os

__all__ = ['process_env']


def process_env(home):
    """Use only declared inputs and the caller-owned temporary home."""
    return {'PATH': os.environ['PATH'], 'HOME': str(home),
            'GIT_CONFIG_GLOBAL': os.devnull, 'GIT_CONFIG_NOSYSTEM': '1'}
