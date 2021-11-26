# ctest_tracing
[![Continuous integration](https://github.com/speedyleion/ctest_tracing/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/speedyleion/ctest_tracing/actions/workflows/rust.yml)

Turn ctest output into a tracing file for chrome's tracing view
## Table of Contents
* [Example Trace](#example-trace)
    - [Example ctest Results](#example-ctest-results)
* [Usage](#usage)
    - [Output](#output)
* [How It Works](#how-it-works)
* [Inspiration](#inspiration)

## Example Trace

The following image is the result of taking the 
[ctest](https://cmake.org/cmake/help/latest/manual/ctest.1.html) results from 
[Catch2 (v2.13.3)](https://github.com/catchorg/Catch2) tests, and feeding it 
through `ctest_tracing`.

![Catch2 Test Tracing](doc/assets/catch2_tests.png)

### Example ctest Results

The [ctest](https://cmake.org/cmake/help/latest/manual/ctest.1.html) results 
which generated the above image.  This can be fed into `ctest_tracing`, as is, 
to generate the same trace info.

    $ ctest -j12
    Test project C:/git/Catch2/bin/msvc
          Start 14: ApprovalTests
          Start  1: RunTests
          Start 24: RandomTestOrdering
          Start  6: NoAssertions
          Start  8: WarnAboutNoTests
          Start  2: ListTests
          Start 18: FilenameAsTagsTest
          Start  7: NoTest
          Start 20: TestsInFile::SimpleSpecs
          Start  3: ListTags
          Start 23: TestsInFile::InvalidTestNames-2
          Start 13: FilteredSection::GeneratorsDontCauseInfiniteLoop-2
     1/24 Test #24: RandomTestOrdering ...................................   Passed    0.93 sec
          Start 12: FilteredSection::GeneratorsDontCauseInfiniteLoop-1
     2/24 Test  #8: WarnAboutNoTests .....................................   Passed    0.94 sec
          Start 21: TestsInFile::EscapeSpecialCharacters
     3/24 Test #18: FilenameAsTagsTest ...................................   Passed    0.90 sec
          Start 10: FilteredSection-1
     4/24 Test  #1: RunTests .............................................   Passed    1.34 sec
          Start 22: TestsInFile::InvalidTestNames-1
     5/24 Test  #7: NoTest ...............................................   Passed    0.96 sec
          Start 15: RegressionCheck-1670
     6/24 Test #20: TestsInFile::SimpleSpecs .............................   Passed    0.92 sec
          Start 11: FilteredSection-2
     7/24 Test  #3: ListTags .............................................   Passed    0.89 sec
          Start 19: EscapeSpecialCharactersInTestNames
     8/24 Test  #6: NoAssertions .........................................   Passed    1.57 sec
          Start  9: UnmatchedOutputFilter
     9/24 Test #23: TestsInFile::InvalidTestNames-2 ......................   Passed    0.96 sec
          Start  5: ListTestNamesOnly
    10/24 Test #13: FilteredSection::GeneratorsDontCauseInfiniteLoop-2 ...   Passed    0.94 sec
          Start 17: LibIdentityTest
    11/24 Test #12: FilteredSection::GeneratorsDontCauseInfiniteLoop-1 ...   Passed    0.94 sec
          Start  4: ListReporters
    12/24 Test #21: TestsInFile::EscapeSpecialCharacters .................   Passed    0.91 sec
          Start 16: VersionCheck
    13/24 Test #10: FilteredSection-1 ....................................   Passed    0.91 sec
    14/24 Test #22: TestsInFile::InvalidTestNames-1 ......................   Passed    0.82 sec
    15/24 Test #15: RegressionCheck-1670 .................................   Passed    0.73 sec
    16/24 Test #11: FilteredSection-2 ....................................   Passed    0.64 sec
    17/24 Test #19: EscapeSpecialCharactersInTestNames ...................   Passed    0.56 sec
    18/24 Test  #9: UnmatchedOutputFilter ................................   Passed    0.47 sec
    19/24 Test #17: LibIdentityTest ......................................   Passed    0.27 sec
    20/24 Test  #4: ListReporters ........................................   Passed    0.21 sec
    21/24 Test  #5: ListTestNamesOnly ....................................   Passed    0.42 sec
    22/24 Test  #2: ListTests ............................................   Passed    2.02 sec
    23/24 Test #16: VersionCheck .........................................   Passed    0.24 sec
    24/24 Test #14: ApprovalTests ........................................   Passed    4.84 sec
    
    100% tests passed, 0 tests failed out of 24
    
    Total Test time (real) =   4.85 sec

## Usage

`ctest_tracing` supports reading ctest results from stdin or a file:

    $ ctest -j12 | ctest_tracing 
    
    $ ctest_tracing output_from_test_run.txt
    
### Output

The output will be a minified json which follows the 
[Chrome trace event format](https://docs.google.com/document/d/1CvAClvFfyA5R-PhYUmn5OOQtYMH4h6I0nSsKchNAySU/preview).

There is the `-o, --output` flag which specifies an output file to write to.  
The output file can be nested and any parent directories will be created, 
i.e. `some/path/to/output.json`.  
> The choice to create parent directories was made based on frustrations of
> having to chain commands with `mkdir` with other tools.  
> If someone has guidance on the downsides of supporting this behavior, 
> feedback would be appreciated.
    
## How it Works

`ctest_tracing` uses a growing queue of identifiers.  Each time ctest 
reports that a test is starting, via `Start #: TestName`, the new test will be 
assigned to the next available identifier.  If the queue is empty then a 
new identifier is created for the test. 

The new test and its identifier is added to a list of running tests.

Each time a test finishes via, 
`#/# Test ##: TestName ..................  Passed    0.20 sec`, it is removed 
from the list of running tests and its identifier is added back to the queue.
When a test finishes that was not in the list of running tests it is ignored.  
Non-existent tests finishing can happen when ctest reports that a test was not 
run.

`ctest_tracing` is a naive implementation which makes some assumptions based on
the output of ctest.  

It is assumed that:

- ctest will start any tests it can immediately after finishing a test.
- ctest's scheduler uses a queue reusing the oldest available job.

## Inspiration

This project was inspired by 
[ninjatracing](https://github.com/nico/ninjatracing). 