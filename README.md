# bulkcp
a command line bulk copying tool

```bash
bulkcp a.txt b.txt
# Copying:
# a.txt -> b.txt

bulkcp '(.+)\\.txt' %1.bak.txt
# Copying:
# a.txt -> a.bak.txt
# b.txt -> b.bak.txt

bulkcp --mv .+ dir/
# Moving:
# a.txt -> dir/a.txt
# b.txt -> dir/b.txt
# a.md -> dir/a.md
# b.md -> dir/b.md

ln -s bulkcp ~/.cargo/bin/bulkmv
bulkmv 'a\\.(?:[^.])+' dir/ --dry-run
# Moving:
# a.txt -> dir/a.txt
# a.md -> dir/a.md
# (filesystem isn't actually touched)
```

I made this in an afternoon because I was bored. You should probably just use rename.