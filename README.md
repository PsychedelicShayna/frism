# Frism (File Prism)

Does what it says on the tin. No dependencies either.

Another CLI utility to split files into arbitrary size chunks, and join them back together again (yes I know a million different archival tools can do this, e.g. 7z but, one tool to do a specific thing and avoid ffmpeg philosophy kicking in here)


```
    Usage: frism <split|join> filename.ext    <N[k|m|g]>    [outfile.ext]
                                              ----------    ------------
                                                (split)        (join)

            Splitting

  $ frism split filename.ext 50m         | All whitespace is eliminated after the
  $ frism split - filename.ext 50m       | filename, and treated as one argument, 
    .................................... | This would still be valid: 1 00 0 000 k 
 >> filename.ext.0, filename.ext.1, etc  | If - is provided as the filename, then
                                         | bytes are read from stdin, and the third
                                         | argument becomes the filename template.

             Joining                      

  $ frism join filename.ext  |  Notice the lack of '.0' at the end; it's no mistake.
    .......................  |  When joining parts, the parts are found automatically
 >> filename.ext             |  by adding 0..inf to the basename until file not found.
                             |  
                             |  The output is then written as the basename. Adding a
                             |  a second filename after the basename will make that 
                             |  the output file, rather than the basename itself.

```
