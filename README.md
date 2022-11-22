Interactive Voronoi
=========================

This is a silly program, written in Rust and using Piston (SDL / OpenGL). It allows you to interactively create Voronoi diagrams.

Original [Demo](https://www.youtube.com/watch?v=5uBe5CkFXlM) version 0.1.0 from ~2017



Command line arguments:
* You can use `-l` to draw lines only, no polygons.
* You can use `-r` to control the number of random dots that appear when you press R.
* You can use `-j` to load a list of points as a json array

Interactive keys:
* Press `N` to clear the screen.
* Press `R` to get _n_ random dots (default 50).
* Press `L` to toggle between wireframe and polygon view
* Press `S` to dump current points to console
