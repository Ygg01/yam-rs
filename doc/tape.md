
# Tape structure in YAM dark core

We parse a YAML document to a tape. A tape is an array of 64-bit values. Each node encountered in the YAML document is written to the tape using one or more 64-bit tape elements; the layout of the tape is in "document order": elements are stored as they are encountered in the YAML document.

Throughout, little endian encoding is assumed. The tape is indexed starting at 0 (the first element is at index 0).

## Example

It is sometimes useful to start with an example. Consider the following YAML document:

```yaml
image:
  width: 600
  height: 800
  thumbnail: {
    url: https://yaml.org,
    height: 125,
    width: 100
  }
  animated: false
  ids: [116, 943, 234, 42]
```

The following is a dump of the content of the tape, with the first number of each line representing the index of a tape element.


### The Tape

| index | discriminant (highest 8bit)        | Rest of element (56 bit)                             |
|-------|------------------------------------|------------------------------------------------------|
| 0     | r                   (stream start) | 41 // pointing to after last node                    |
| 1     | { + 128 = û   (implicit map start) | 40 // pointing to first node after the scope         |
| 2     | s                (string: "image") | 0  // location of the string "image" on string stack |
| 3     |                                    | 5  // previous location + length of string           |
| 4     | { + 128 = û   (implicit map start) | 37 // pointing to first node after the scope         |
| 5     | s                (string: "width") | 5  // location of the string "width" on string stack |
| 6     |                                    | 10 // previous location + length of string           |
| 7     | u          (unsigned integer: 600) |                                                      |
| 8     |                                    | 600  // unsigned integer value                       |
| 5     | s               (string: "height") |                                                      |
| 7     | u          (unsigned integer: 800) |                                                      |
| 9     | u            (string: "thumbnail") |                                                      |
| 11    | {                                  | 25 // pointing to first node after scope             |
| 12    | s                  (string: "url") |                                                      |
| 14    | s     (string: "https://yaml.org") |                                                      |
| 16    | s               (string: "height") |                                                      |
| 18    | u          (unsigned integer: 125) |                                                      |
| 20    | s                (string: "width") |                                                      |
| 22    | u          (unsigned integer: 100) |                                                      |
| 24    | }                                  | 11 // location of the string "image" on string stack |
| 25    | s             (string: "animated") |                                                      |
| 27    | f                 (boolean: false) |                                                      |
| 28    | [        (explicit sequence start) | 38 // pointing to first node after sequences         |
| 29    | u          (unsigned integer: 116) |                                                      |
| 31    | u          (unsigned integer: 943) |                                                      |
| 33    | u          (unsigned integer: 234) |                                                      |
| 35    | u           (unsigned integer: 42) |                                                      |
| 37    | ]                                  | 28 // pointing to start of sequence                  |
| 38    | }                                  | 4 // location of the string "image" on string stack  |
| 39    | }                                  | 1 // location of the string "image" on string stack  |
| 40    | r                                  | 0  // pointing to 0 (start root)                     |




## General formal of the tape elements

Most tape elements are written as `('c' << 56) + x` where `'c'` is some character determining the type of the element (out of `t`, `f`, `n`, `l`, `u`, `d`, `"`, `{`, `}`, `[`, `]` ,`r`, `&`, `*`, `!` and `%`) and where `x` is a 56-bit value called the payload. The payload is normally interpreted as an unsigned 56-bit integer. NOTE: There are special events that are emitted only for YAML event compatibility - `>`, `|`, `'`, `s`, `û` (binary OR of `{` and 128) , `Û` (binary OR of `[` and 128), `ò`(binary OR of `r` and 128).



## Simple YAML values

Simple YAML nodes are represented with one tape element:

- null is  represented as the 64-bit value `('n' << 56)` where `'n'` is the 8-bit code point values (in ASCII) corresponding to the letter `'n'`.
- true is  represented as the 64-bit value `('t' << 56)`.
- false is  represented as the 64-bit value `('f' << 56)`.


## Integer and Double values

Integer values are represented as two 64-bit tape elements:
- The 64-bit value `('l' << 56)` followed by the 64-bit integer value literally. Integer values are assumed to be signed 64-bit values, using two's complement notation.
- The 64-bit value `('u' << 56)` followed by the 64-bit integer value literally. Integer values are assumed to be unsigned 64-bit values.


Float values are represented as two 64-bit tape elements:
- The 64-bit value `('d' << 56)` followed by the 64-bit double value literally in standard IEEE 754 notation.

Performance consideration: We store numbers of the main tape because we believe that locality of reference is helpful for performance.

## Root node

Each YAML document will have two special 64-bit tape elements representing a root node, one at the beginning and one at the end.

- The first 64-bit tape element contains the value `('r' << 56) + x` where `x` is the location on the tape of the last root element.
- The last 64-bit tape element contains the value `('r' << 56)`.

All of the parsed document is located between these two 64-bit tape elements.

Hint: We can read the first tape element to determine the length of the tape.


## Strings

We prefix the string data itself by a 32-bit header to be interpreted as a 32-bit integer. It indicates the length of the string. The actual string data starts at an offset of 4 bytes.

We depart in representing string values as three possible values `'`

## Sequences

YAML  arrays are represented using two 64-bit tape elements.

- The first 64-bit tape element contains the value `('[' << 56) + (c << 32) + x` where the payload `x` is 1 + the index of the second 64-bit tape element on the tape  as a 32-bit integer and where `c` is the count of the number of elements (immediate children) in the array, satured to a 24-bit value (meaning that it cannot exceed 16777215 and if the real count exceeds 16777215, 16777215 is stored).  Note that the exact count of elements can always be computed by iterating (e.g., when it is 16777215 or higher).
- The second 64-bit tape element contains the value `(']' << 56) + x` where the payload `x` contains the index of the first 64-bit tape element on the tape.

All the content of the array is located between these two tape elements, including arrays and objects.

Performance consideration: We can skip the content of an array entirely by accessing the first 64-bit tape element, reading the payload and moving to the corresponding index on the tape.

## Maps

YAML maps are represented using two 64-bit tape elements.

- The first 64-bit tape element contains the value `('{' << 56) + (c << 32) + x` where the payload `x` is 1 + the index of the second 64-bit tape element on the tape as a 32-bit integer and where `c` is the count of the number of key-value pairs (immediate children) in the array, satured to a 24-bit value (meaning that it cannot exceed 16777215 and if the real count exceeds 16777215, 16777215 is stored). Note that the exact count of key-value pairs can always be computed by iterating (e.g., when it is 16777215 or higher).
- The second 64-bit tape element contains the value `('}' << 56) + x` where the payload `x` contains the index of the first 64-bit tape element on the tape.

In-between these two tape elements, we alternate between key (which must be strings) and values. A value could be an object or an array.

All the content of the object is located between these two tape elements, including arrays and objects.

Performance consideration: We can skip the content of an object entirely by accessing the first 64-bit tape element, reading the payload and moving to the corresponding index on the tape.

## Tags

YAML tags are represented by one 64-bit tape element. When they appear they are applied to next element in the YAML.

- It is  represented as the 64-bit value `('!' << 56) + (c << 32) + x` where `!` is the 8-bit code point values (in ASCII) corresponding to the character `'!'`, `c` is the 0 index in the namespace stack and x is length of the namespace.

## Anchor

YAML anchor are represented by one 64-bit tape element. When they appear they are applied to next element in the YAML.

- It is represented as the 64-bit value `('&' << 56) + (c << 32) + x` where `&` is the 8-bit code point values corresponding to character `&`, `c` is the 0 index in the alias namespace stack and x