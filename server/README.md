# NavigaTUM-server

This repository contains the backend server for NavigaTUM.

## Getting started

### 0. Install Rust

Unless you haven't already, you need to [install Rust](https://www.rust-lang.org/tools/install)
in order to compile and run this server.

If you want to run the tests you need at least Python 3.6 as well.

### 1. Get the data

The data is provided to the server with just a simple JSON file.
You can create a `data` subdirectory and copy the `api_data.json`
(and optionally `search_data.json`) file into it.

Alternatively, link the output directory to the server data directory,
so that you don't need to copy on every update:

```bash
ln -s ../data/output data
```

### 2. Starting the server

Run

```bash
cargo run --release
```

The server should now be available on `localhost:8080`.

### 3. Setup MeiliSearch (optional)
The server uses [MeiliSearch](https://github.com/meilisearch/MeiliSearch) as a backend for search.
For a local test environment you can skip this step if you don't want to test or work on search.

There are a lot of different ways to run MeiliSearch (see on their repo). Here we compile it
from sources:

```bash
# Clone MeiliSearch
cd ..
git clone https://github.com/meilisearch/MeiliSearch.git -b v0.22.0
cd MeiliSearch

# Build and run
cargo run --release
```

Next, we need to add our index and configure search:

```bash
# Create index
curl -i -X POST 'http://localhost:7700/indexes' --header 'content-type: application/json' --data '{ "uid": "entries", "primaryKey": "ms_id" }'

# Set filterable attributes
curl -X POST 'http://localhost:7700/indexes/entries/settings/filterable-attributes' --data '["facet"]'

# Upload entries data
curl -i -X PUT 'http://localhost:7700/indexes/entries/documents' --header 'content-type: application/json' --data-binary @data/search_data.json

# Configure index
curl -X POST 'http://localhost:7700/indexes/entries/settings/ranking-rules' --data '["words","typo","rank:desc","exactness","proximity","attribute"]'

curl -X POST 'http://localhost:7700/indexes/entries/settings/synonyms' --data @../data/search_synonyms.json

curl -X POST 'http://localhost:7700/indexes/entries/settings/searchable-attributes' --data '[ "ms_id", "name", "arch_name", "type", "type_common_name", "parent_building", "parent_keywords", "address", "usage" ]'
```

If you want to update the data in the index, run:

```bash
curl -i -X PUT 'http://localhost:7700/indexes/entries/documents' --header 'content-type: application/json' --data-binary @data/search_data.json
```

And if you want to delete the index, run:

```bash
curl -X DELETE 'http://localhost:7700/indexes/entries'
```

MeiliSearch provides an interactive interface at [http://localhost:7700](http://localhost:7700).

## API

### get

```HTTP
GET https://nav.tum.sexy/api/get/:id
```

This returns the full data available for the entry (room/building) available as JSON.
The exact data format is specified in the [NavigaTUM-data documentation](../data/README.md), but it is essentially structured like this:

e.g. with `GET https://nav.tum.sexy/api/get/5602.EG.001`:

```js
{
    "id": "5602.EG.001",
    "type": "room",  // or "building", "joined_building",  ...
    "type_common_name": "Hörsaal",
    "name": "5602.EG.001 (MI HS 1, Friedrich L. Bauer Hörsaal)",
    "parent_names": ["Standorte", "Garching Forschungszentrum", ...],
    "parents": ["root", "garching", ...],  // IDs to the names above

    "coords": {
        "lat": 48.26244490906312,
        "lon": 11.669122601167174,
        "source": "roomfinder"
    },
    "maps": {
        "default": "interactive",  // else: "roomfinder"
        "roomfinder": {
            "available": [
                {
                    "name": "FMI Übersicht",
                    "id": 142,
                    "scale": "2000",  // Scale 1:2000
                    "height": 461,  // Map image dimensions
                    "width": 639,
                    "x": 499,  // Position on map image
                    "y": 189
                },
                ...
            ],
            "default": "142"
        }
    },

    "props": {
        "computed": [  // Date for the info-card table
            {
                "name": "Raumkennung",
                "text": "5602.EG.001"
            },
            ...
        ]
    },
    
    img: {
      "name": "mi_0.webp" // The name of the image file.
      // consists of {building_id}_{image_id}.webp, where image_id is a counter starting at 0
      // You can request images by their name with 
      
      // GET /cdn/:size/:name
      // e.g. GET https://nav.tum.sexy/cdn/lg/mi_0.webp
        
      // Availible image sizes:
      // :size   spec
      // lg      max 4k, aspect ratio untouched
      // md      max 1920px, aspect ratio untouched
      // md      max 1024px, aspect ratio untouched
      // thumb   256x256, cropped to fit. Usially a center-crop, but sometimes offset.
      // header  512x210, cropped to fit. Usially a center-crop, but sometimes offset.
        
      // for each: text is required, url can also be null
      "author": {
        "text": "...",
        "url": "...",
      },
      "source": {
        "text": "...",
        "url": "...",
      },
      "license": {
        "text": "...",
        "url": "...",
      },

      "meta": {          // optional
        "date": "",      // optional
        "location": "",  // optional location description
        "geo": "",       // optional coordinates in lat,lon
        "image_url": "", // optional, in contrast to source this points to the image itself
        "caption": "",   // optional
        "headline": "",  // optional
        "event": "",     // optional, the event this image was taken at
        "faculty": "",   // optional
        "building": "",  // optional
        "department": "",// optional
      }
    },
    "sections": {
        // Info sections with more details, currently only for buildings etc.
    }
}
```

### search

```HTTP
GET https://nav.tum.sexy/api/search/:query
```

Search entries – this endpoint is designed to support search-as-you-type results.

Instead of simply returning a list, the search results are returned in a way to provide
a richer experience by splitting them up into sections. You might not necessarily need to
implement all types of sections, or all sections features (if you just want to show a list).
The order of sections is a suggested order to display them, but you may change this as
you like.

Some fields support highlighting the query terms and it uses DC3 (`\x19` or `\u{0019}`) and
DC1 (`\x17` or `\u{0017}`) to mark the beginning/end of a highlighted sequence
([See Wikipedia](https://en.wikipedia.org/wiki/C0_and_C1_control_codes#Modified_C0_control_code_sets)).
Some text-renderers will ignore them, but in case you do not want to use them, you might want
to remove them from the responses (there is no query parameter for this yet).

The general response format looks like this:

```js
{
    "sections": [{ see below }, ...],
    "time_ms": 7  // Time the search took in the server side
}
```

The following sections are currently implemented:

**Rooms:**

```js
{
    "facet": "rooms",
    "entries": [
        {
            "id": "5502.01.250",
            "type": "room",
            // Name of this search result, highlighted
            "name": "5502.01.\u0019250\u0017 (Hörsaal)",
            // Subtext to show below the search result. Usually contains
            // the context of where this rooms is located in.
            // Currently not highlighted.
            "subtext": "Maschinenwesen (MW)",
            // Subtext to show below the search (by default in bold and after the
            // non-bold subtext). Usually contains the arch-id of the room, which is
            // another common room id format, and supports highlighting.
            "subtext_bold": "1\u0019250\u0017@5502",

            // This is an optional feature, that is only supported for some rooms.
            // It might be displayed instead or before the name, to show that a
            // different room id format has matched, that was probably used.
            // See the image below for an example.
            // Supports highlighting.
            "parsed_id": "\u0019MW 250\u00171"
        }
    ],
    "nb_hits": 30  // The estimated (not exact) number of hits for that query
}
```

Example of how `parsed_id` might be displayed:  
![example of displaying parsed_id](../resources/website-screenshots/example_parsed-id_light.png#gh-light-mode-only)
![example of displaying parsed_id](../resources/website-screenshots/example_parsed-id_dark.png#gh-dark-mode-only)

**Buildings / Sites:**

```js
{
    "facet": "sites_buildings",
    "entries": [
        {
            "id": "mw",
            "type": "joined_building",
            // Name of this search result, highlighted
            "name": "Maschinenwesen (\u0019MW\u0017)",
            // Subtext to show below the search result. Usually contains
            // the what type of result this is.
            // Currently not highlighted.
            "subtext": "Gebäudekomplex"
        }
    ],
    "n_visible": 5,  // How many of the above entries should be displayed by default. The number is usually from 0-5.
                     // More results might be displayed when clicking "expand".
    "nb_hits": 19  // The estimated (not exact) number of hits for that query
}
```

#### Query parameters

Limits are not stable yet.

| name              | default | description                                 |
|-------------------|---------|---------------------------------------------|
| `limit_buildings` | 5       | Maximum number of buildings/sites to return |
| `limit_rooms`     | 5       | Maximum number of rooms to return           |
| `limit_all`       | 20      | Overall maximum number of results           |

### source_code

```HTTP
GET https://nav.tum.sexy/api/source_code
```

The `api/source_code` endpoint returns a link to the source-code of the
repository at the currently running version.  
This is not required for modifications (as the license
is not AGPL), but strongly encouraged.


## Feedback

### get_token

```HTTP
POST https://nav.tum.sexy/api/feedback/get_token
```

***Do not abuse this endpoint.***

You should request a token, ***if (and only if) a user is on a feedback page***

As a rudimentary way of rate-limiting feedback, this endpoint returns a token. 
To post feedback, you will need this token. 

Tokens gain validity after 10s, and are invalid after 12h of being issued.
They are not refreshable, and are only valid for one usage.

Global Rate-Limiting:
 - hourly: 20 tokens per hour
 - daily: 50 tokens per day

A token is returned via a `201 Created` response in the body in the following json format:

```json
{
    "token": "999999999999999"
}
```

Errors:
- `429`: Too many requests. We are rate-limiting everyone's requests, please try again later.
- `503`: Service unavailable, if the token could not be generated

### send feedback

```HTTP
POST https://nav.tum.sexy/api/feedback/feedback
```
Post-Data (all fields are required):
```ts
{
  token = "999999999999999", 
    // The token optained by the get_token endpoint as shown above
  category = "bug",  
    // The category of the feedback.
    // One of: "general", "bug", "feature", "search", "entry", "other"
  subject = "A catchy title", 
    // The subject/title of the feedback
    // min 4 chars, max 512 chars
  body = "A clear desription what happened where and how we should improve it",
    // The body/desription of the feedback.
    // min 4 chars, max 1024*1024 chars
  privacy_checked = true,
    // Whether the user has checked the privacy-checkbox. 
    // We are posting the feedback publicly on GitHub (not a EU-Company). You have to also include such a checkmark.
    // For inspiration on how to do this, see our website.
  delete_issue_requested = true
    // Whether the user has requested to delete the issue.
    // If the user has requested to delete the issue, we will delete it from GitHub after processing it
    // If the user has not requested to delete the issue, we will not delete it from GitHub and it will remain as a closed issue.
}
```

Reruns:
a link to the GitHub issue is returned via a `201 Created` response.

Errors:
- `400`: If not all fields are present as defined above
- `403`: Forbidden:
  Causes (delivered via the body):
    - `Invalid token`: You have not supplied a token generated via the `gen_token`-Endpoint.
    - `Token not old enough, please wait.`: Tokens are only valid after 10s.
    - `Token expired.`: Tokens are only valid for 12h.
    - `Token already used.`: Tokens are non reusable/refreshable single-use items.
- `422`: Unprocessable Entity: `Subject or body missing or too short.` 
- `451`: Unavailable for legal reasons. Using this endpoint without accepting the privacy policy is not allowed. For us to post to GitHub, this has to be true
- `500`: Internal Server Error. We have a problem communicating with GitHubs servers. Please try again later.
- `503`: Service unavailable. We have not configured a GitHub Access Token. This could be because we are experiencing technical difficulties or intentional. Please try again later.

***Important Note:*** Tokens are only used if we return a `201 Created` response. Otherwise, they are still valid

## License

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.

---
