import re
import pathlib

# Get filters
my_filters = [l.rstrip('\n') for l in open('.dockerignore') if l[0] is not '#' and l is not '']

def matches_filters(filters, file):
    for filter in filters:
        if re.search(filter, file):
            return True
    return False

# Get all the files that don't match the filters
files = [str(i) for i in pathlib.Path().rglob("*") if i.is_file() and not matches_filters(my_filters, str(i))]

# Print out the excluded files
print(' '.join(files))
