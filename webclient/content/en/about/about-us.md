---
title: About NavigaTUM
description: NavigaTUM is an open-source tool developed by students for students to help you find your way around the Technical University of Munich (TUM).
---

# About NavigaTUM

NavigaTUM helps you find rooms, buildings, and locations at [TUM](https://tum.de) - from the main campus in Munich to Garching, Weihenstephan, Heilbronn, and Straubing.
The project is developed by students for students, is [open source](https://github.com/TUM-Dev/NavigaTUM), and works without a login, without ads, and without tracking.
It is not run by an official TUM department, but by a group of volunteers at [OpenSource @ TUM e.V.](https://tum.dev) - and maybe soon by you.

## Where the data comes from

Good navigation lives and dies with good data, so we combine several sources.
The foundation are the buildings, rooms, organisations, and calendar entries from [TUMonline](https://campus.tum.de).
We import this data regularly and automatically, but partially restructure it (for example for building complexes) and fix errors.
We replaced the old TUM Roomfinder; its site plans and room metadata live on in NavigaTUM as an archived dataset.
On top of that, we curate a lot by hand, often based on hints from the community: coordinates, names, the search ranking, opening hours with a source reference per entry, and some hand-drawn site plans.

The backbone of our map is [OpenStreetMap](https://www.openstreetmap.org).
Building outlines, indoor data, points of interest, and many opening hours come from OSM - and it is not a one-way street: improvements we make on the ground go back into OSM, so every other map benefits too.
Map data © OpenStreetMap contributors.

Some content is live.
Public transport stops near the campuses and the departures on detail pages come from the community-run public transport routing project [Transitous](https://transitous.org), and the current availability of study rooms from [IRIS](https://iris.asta.tum.de), the student council's study room display.
The menus and opening hours of the canteens are provided by the [eat-api](https://github.com/TUM-Dev/eat-api), derived from the data of the Studierendenwerk München Oberbayern; the information about the branch libraries comes from the [TUM University Library](https://www.ub.tum.de/en/branch-libraries).

## Images

The photos of buildings and rooms come from members of the community.
For every image, we record who took it and under which license it is published (for example CC0 or CC BY) - you can view this attribution directly on the image.
Been out with your camera? You can contribute your own photos via the edit button on every detail page.

## How to contribute

NavigaTUM thrives on people reporting errors and filling gaps.
The easiest way is the feedback form on every page - it creates a public issue [on GitHub](https://github.com/TUM-Dev/NavigaTUM/issues) whose progress you can follow.
You can fix wrong coordinates or names directly on the detail page and [propose](/en/propose) missing entries.
If you prefer working on the map, add missing details directly to [OpenStreetMap](https://www.openstreetmap.org) - they automatically show up here as well.
And if you want to help build NavigaTUM itself: [the code is open source](https://github.com/TUM-Dev/NavigaTUM) and we are happy to welcome new contributors.
