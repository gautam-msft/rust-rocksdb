//  Copyright (c) 2013, Facebook, Inc.  All rights reserved.
//  This source code is licensed under the BSD-style license found in the
//  LICENSE file in the root directory of this source tree. An additional grant
//  of patent rights can be found in the PATENTS file in the same directory.
//
#pragma once
#include <string>
#include <unordered_map>
#include <memory>

#ifndef ROCKSDB_NAMESPACE
#define ROCKSDB_NAMESPACE rocksdb
#endif

#define ROCKSDB_MAJOR 6
#define ROCKSDB_MINOR 18
#define ROCKSDB_PATCH 0

// Do not use these. We made the mistake of declaring macros starting with
// double underscore. Now we have to live with our choice. We'll deprecate these
// at some point
#define __ROCKSDB_MAJOR__ ROCKSDB_MAJOR
#define __ROCKSDB_MINOR__ ROCKSDB_MINOR
#define __ROCKSDB_PATCH__ ROCKSDB_PATCH

template <typename T>
inline std::string ToString(T value) {
#if !(defined OS_ANDROID) && !(defined CYGWIN) && !(defined OS_FREEBSD)
  return std::to_string(value);
#else
  // Andorid or cygwin doesn't support all of C++11, std::to_string() being
  // one of the not supported features.
  std::ostringstream os;
  os << value;
  return os.str();
#endif
}

namespace ROCKSDB_NAMESPACE {
// Returns a set of properties indicating how/when/where this version of RocksDB
// was created.
const std::unordered_map<std::string, std::string>& GetRocksBuildProperties();

// Returns the current version of RocksDB as a string (e.g. "6.16.0").
// If with_patch is true, the patch is included (6.16.x).
// Otherwise, only major and minor version is included (6.16)
std::string GetRocksVersionAsString(bool with_patch = true);

// Gets the set of build properties (@see GetRocksBuildProperties) into a
// string. Properties are returned one-per-line, with the first line being:
// "<program> from RocksDB <version>.
// If verbose is true, the full set of properties is
// printed. If verbose is false, only the version information (@see
// GetRocksVersionString) is printed.
std::string GetRocksBuildInfoAsString(const std::string& program,
                                      bool verbose = false);
}  // namespace ROCKSDB_NAMESPACE