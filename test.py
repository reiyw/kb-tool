import sys

from kb_tool import PathSampler

sampler = PathSampler(sys.argv[1], 1.5, 3, 810)
for _ in range(10):
    print(sampler.sample_path_with_negative())
