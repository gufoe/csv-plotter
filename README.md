# Real time csv plotter

## Install
`cargo install --git https://github.com/gufoe/csv-plotter.git --branch main`

## Usage
```
$ csv-plotter --help
Giacomo R. <gufoes@gmail.com>

USAGE:
    csv-plotter [OPTIONS] [data]...

ARGS:
    <data>...    

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -s, --separator <separator>    [default: ,]
    -t, --title <title>            
    -x, --x <x>                    
    -y, --y <y>                    [default: 1]

```


## Examples
Plot file using first column for X axis and second column for Y axis  
`csv-plotter -x 0 -y 1 ~/.local/atom.log`

Plot file using row number for X axis and first column for Y axis in a tab separated file  
`csv-plotter -y 0 --separator '\t' /tmp/test.log`
