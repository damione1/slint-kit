#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Prepare the exact sources used by downloadable projects and native builds."""
import argparse

from starter_sources import BuildMode, prepare

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--mode', choices=[mode.value for mode in BuildMode])
args = parser.parse_args()
prepare(args.mode)
