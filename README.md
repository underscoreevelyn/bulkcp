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

bulkcp '(a|b)/(a.+)' %1-%2
# (does not match anything)
bulkcp -r '(a|b)/(a.+)' %1-%2
# Copying:
# a/a.txt -> a-a.txt
# b/a.txt -> b-a.txt

bulkcp .+ %U0
# Copying:
# a.txt -> A.TXT
# b.txt -> B.TXT

bulkcp .+ %L0
# Copying:
# README.md -> readme.md

bulkcp '(.+)-(.+)' %C1-%C2
# Copying:
# inconvenient-file.txt -> Inconvenient-File.txt
```

I made this in an afternoon because I was bored. You should probably just use rename.  

i wonder what happens when you feed this thing a symlink?
