# glompack

unoptimized png packer with an index file

## what it does

1. reads png
2. writes png data to `.gpk` file and compresses the png
3. writes data required for getting the png file back in an index file `.gdx`
 - in the format of `filename png-data-size png-data-start-offset`