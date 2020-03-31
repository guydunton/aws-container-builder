import glob
import os
import re

# Get filters
my_filters = [l.rstrip('\n') for l in open('.dockerignore') if l[0] is not '#' and l is not '']

def matches_filters(filters, file):
    for filter in filters:
        if re.search(filter, file):
            return True
    return False

# Get all the files that don't match the filters
files = [i for i in glob.glob('./**', recursive=True) if os.path.isfile(i) and not matches_filters(my_filters, i)]

# Remove the ./ from the start of each file
files = [file[2:] for file in files]

# Print out the excluded files
print(' '.join(files))
