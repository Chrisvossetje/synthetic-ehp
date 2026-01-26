# Synthetic EHP

Displays the synthetic EHP. Also allows for view AEHP and normal EHP and its truncations.

Curtis table from William Balderamma's website.
Display based on my Ext resolver which is in turn based on the sseq project / d3.

# How to run

## Prerequisites
- npm 
- python

## Run

To build the site run:
- npm run build

To continuously build the site run:
- npm run watch

Note that this only watches changes in site/src, NOT in site/static

The site is now statically build in the folder: _site

Because of CORS reasons you need to host the site using f.e. python3 as follows:
- cd ./_site && python3 -m http.server 8080

Then go the browser and browse to:
- localhost:8080

## What actually happens
Before each tsc build we remove the whole _site folder and copy all static files to _site again.
Then we use ESBuild to bundle and compile all our typescript files into a single index.js file (possibly minified) which also gets copied to _site