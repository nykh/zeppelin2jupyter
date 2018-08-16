Converts a zeppelin json to a jupyter json

# Transform rules

## Overall structure

Zeppelin notebook structure
```json
{
  "paragraphs": [...],
  "name": "",
  "id": "",
  "noteParams": {},
  "noteForms": {},
  "angularObjects": { ... },
  "config": { ... },
  "info": {}
}
```
Zeppelin keeps around a lot more meta data, which are however irrelevant and can be discarded.

Jupyter notebook structure 
```json
{
  "cells": [...],
  "metadata": { ... },
  "nbformat": 4,
  "nbformat_minor": 2
}
```
the `metadata` is mostly constant and irrelevant to rendering.

## Transform a cell

### Jupyter cells

Jupyter has two kinds of cells: a code cell and a markdown cell
```json
{
  "cell_type": "code",
  "execution_count": null,
  "metadata": {},
  "outputs": [...],
  "source": [...]
}
```
where `source` is one string for each line of code (`\n`-terminated); `outputs` is a list whose elements can be

**stream**
```json
{
  "name": "stdout",
  "output_type": "stream",
  "text": [...]
}
```

**display**
```json
{
  "data": {
    "image/png": "<encoded image stream>",
    "text/plain": ["<Figure size 936x288 with 2 Axes>"]
  },
  "metadata": {},
  "output_type": "display_data"
}
```

**execution result** which is the most generic one and include widgets
```json
{
  "data": {
    "text/html": ["<html repr>"],
    "text/plain": ["<Figure size 936x288 with 2 Axes>"]
  },
  "metadata": {},
  "output_type": "display_data"
}
```
where `text/html` is a list of HTML strings (`\n`-terminated)

Finally, there is also a markdown cell
```json
{
  "cell_type": "markdown",
  "metadata": {},
  "source": [...]
}
```

### Zeppelin cells

Zeppelin also has two kinds of cells: code cell and text cell

A typical code cell in Zeppelin is much more complex

```json
{
  "title": "title",
  "text": "a single multi-line string",
  "user": "...",
  "dateUpdated": "...",
  "config": { ... },
  "settings": {
    "params": {},
    "forms": {}
  },
  "results": {
    "code": "SUCCESS",
    "msg": [...]
  },
  "apps": [],
  "jobName": "...",
  "id": "...",
  "dateCreated": "",
  "dateStarted": "",
  "dateFinished": "",
  "status": "FINISHED",
  "progressUpdateIntervalMs": 500,
  "focus": true,
  "$$hashKey": ""
}
```

From this we can see Zeppelin is much more geared towards collaborative notebook. But for rendering we only just need the source and result. The `msg` list in result can contain some different elements. `title` is a special feature of Zeppelin that doesn't have equivelent for Jupyter, the best approximation would be to insert a text cell just before the code cell.

**TEXT**

It is worth noting that Zeppelin notebook likes to output intermediate results, which gets very verbose.

```json
{
  "type": "TEXT",
  "data": "a single multi-line string"
}
```

**HTML**
```json
{
  "type": "HTML",
  "data": "a single multi-line HTML representation"
}
```

Another use for HTML is to display inline image in base64 encoding
```json
{
  "type": "HTML",
  "data": "<div style='width:auto;height:auto'><img src=data:image/png;base64,... style='width=auto;height:auto'><div>\n"
}
```

**TABLE**

After inspecting the raw JSON file, the TABLE display method seems more sophisticated than what is described on [the official documentation](https://zeppelin.apache.org/docs/0.8.0/usage/display_system/basic.html#table). It seems for TABLEs, the config data is needed to disambiguate on the column names. Here is an example

```json
{
  "title": "title",
  "text": "...",
  "user": "",
  "dateUpdated": "",
  "config": {
    "colWidth": 12,
    "fontSize": 9,
    "enabled": true,
    "results": {
      "1": {
        "graph": {
          "mode": "table",
          "height": 300,
          "optionOpen": false,
          "setting": {
            "table": {
              "tableGridState": {
                "columns": [...],
                "scrollFocus": {},
                "selection": [],
                "grouping": {
                  "grouping": [],
                  "aggregations": [],
                  "rowExpandedStates": {}
                },
                "treeView": {},
                "pagination": {
                  "paginationCurrentPage": 1,
                  "paginationPageSize": 250
                }
              },
              "tableColumnTypeState": {
                "names": {
                  "A": "string",
                  "B": "string",
                  "C": "string",
                  "D": "string",
                  "E": "string",
                  "F": "string",
                  "G": "string",
                  "H": "string",
                  "I": "string",
                  "J": "string",
                  "K": "string",
                  "L": "string",
                  "M": "string",
                  "N": "string",
                  "O": "string",
                  "P": "string"
                },
                "updated": false
              },
              "tableOptionSpecHash": "...",
              "tableOptionValue": { ... },
              "updated": false,
              "initialized": false
            }
          },
          "commonSetting": {}
        }
      }
    },
    "editorSetting": { ... },
    "editorMode": "ace/mode/scala"
  },
  "settings": {
    "params": {},
    "forms": {}
  },
  "results": {
    "code": "SUCCESS",
    "msg": [
      {
        "type": "TEXT",
        "data": "df: org.apache.spark.sql.DataFrame = [timestamps: array<bigint>, values: array<double> ... 14 more fields]\n"
      },
      {
        "type": "TABLE",
        "data": "A B C D E F G H I J K L M N O P\n
        WrappedArray(1, 2, 3) WrappedArray(1, 2, 3) foo bar foo bar foo bar foo baz foo foo baz bat goo foo 001 foo foo x fop\n
        ..."
      },
      {
        "type": "HTML",
        "data": "<div class=\"result-alert alert-warning\" role=\"alert\"><button type=\"button\" class=\"close\" data-dismiss=\"alert\" aria-label=\"Close\"><span aria-hidden=\"true\">&times;</span></button><strong>Output is truncated</strong> to 102400 bytes. Learn more about <strong>ZEPPELIN_INTERPRETER_OUTPUT_LIMIT</strong></div>"
      }
    ]
  },
  "apps": [],
  "jobName": "",
  "id": "",
  "dateCreated": "",
  "dateStarted": "",
  "dateFinished": "",
  "status": "FINISHED",
  "progressUpdateIntervalMs": 500,
  "$$hashKey": ""
}
```

What's important in this example:

1. Everything is space-delimited, contrary to the documentation. The exception being `WrappedArray`, which, when encountered, needs to be considered as one whole
2. `config.results.1.graph.setting.table.tableColumnTypeState.names` provides a dictionary of columns mapped to data type. This allows us to disambiguate some column names that may contain space

**Markdown** (`%md`)

Another type of cells are Markdown cells. In Zeppelin markdown cells are in fact just code cells that use a Markdown interpreter
```json
{
  "text": "%md\n# Exploration",
  "user": "...",
  "dateUpdated": "...",
  "config": { ... },
  "settings": {
    "params": {},
    "forms": {}
  },
  "results": {
    "code": "SUCCESS",
    "msg": [
      {
        "type": "HTML",
        "data": "<div class=\"markdown-body\">\n rendered markdown \n</div>"
      }
    ]
  },
  "apps": [],
  "jobName": "",
  "id": "",
  "dateCreated": "",
  "dateStarted": "",
  "dateFinished": "",
  "status": "FINISHED",
  "progressUpdateIntervalMs": 500,
  "$$hashKey": ""
}
```

For these kinds of cell we can just detect that they are markdown cells and use the raw `text` to make a Markdown cell in Jupyter.