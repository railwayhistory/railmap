# railmap

This repository contains the map renderer behind the Railway History Map.
It takes a map described through a simple language based on a set of
paths, renders them as bitmap tiles, and provides those via a built-in
HTTP server.

The map resulting from the description in the sibling
[railwayhistory/map](https://github.com/railwayhistory/map) repository can
be viewed at [map.railwayhistory.org](https://map.railwayhistory.org/).

## Running locally

You can run the renderer locally. For this you need a local copy of the
[map definition](https://github.com/railwayhistory/map) and point to its
map configuration `config.toml` using the `-m` option.

You can limit the regions rendered by specifying the ones you want with
the `-r` option.  This is mostly helpful to decrease the startup time
during map editing.  The available regions are given in the map
configuration file.

By default, the renderer will listen on `127.0.0.1:8080` but you can
change this through the `-l` option. It provides a simple debug view of
the map, so, you can simply point your browser to the address, e.g.,
`http://127.0.0.1:8080/` if you haven’t changed the default.


## Available Layers

The renderer provides an HTTP server that provides multiple layers.
Each produces tiles in spherical mercator projection using the
usual OSM tile layer convention for addressing tiles using _z,_ _x,_ and
_y_ coordinates. The convention is

```
/{layer}/{z}/{x}/{y}.png
```

(It can also produces SVG tiles by replacing `.png` with `.svg` but the
results are likely going to look a bit odd).

The following layers are currently available:

*  `el`: railway lines colored according to their electrification scheme,
*  `el-lat`: the `el` layer but with names transliterated into Latin
   script,
*  `el-num`: Railway History Database line number colored for use with the
   `el` layer,
*  `pax`: railway lines colored according to the passenger service
   provided,
*  `pax-lat`: the `pax` layer but with names transliterated into Latin
   script,
*  `pax-num`: timetable line numbers,
*  `border`: borders contours.

The rendered map publishes these layers at
`https://map.railwayhistory.org/rail/`. E.g., if you want to use the
_el_ layer, the usual configuration string is
`https://map.railwayhistory.org/rail/el/{z}/{x}/{y}.png`.
