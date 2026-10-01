# Example

```md
"Deals **{blue:{damage_number}}** {red:fire} damage to the enemy."
```

# Syntax

## Style

```md
{style:value}
```

This is equivalent to `<style>value</style>` in html.
The left hand side is the name of the style, it will be passed to the `stylesheet` function.

`value` will be trimmed of whitespace, so the correct syntax is to add whitespace outside of the braces:

```md
Hello {style:World}! // Hello World!
Hello{style: World}! // HelloWorld!
```

Style commands also can be chained:

```md
Deals {red, s-black, s-10: 10} damage!
```

## Standard Styles

These will be parsed regardless of the `stylesheet` function:

* `red` Parses Css color names as fill color.
* `#ff00ff` Parses hex color (accepts 3, 4, 6, 8 digits) as fill color.
* `s-4` Sets stroke to a number.
* `s-red` Parses color names as stroke color.
* `v-4.0` Sets the `magic_number` field.
* `f-Roboto` Sets the font to Roboto.
* `$18` Sets font size to `18`.
* `*1.5` Sets font size to `1.5` times the original.
* `h1` - `h4` Sets font size to `2`, `1.75`, `1.5`, `1.25` times the original.

## Dynamic value

```md
{ value }
```

Without `:` values in brackets are treated as dynamic values and passed to the `fetch_string` function.
The result should either be a string fetched from the world
or an [`Entity`](bevy::ecs::entity::Entity) with a [`FetchedTextSegment`](crate::FetchedTextSegment) component.

## Conditions

```md
{ ?condition : value }
{ ?!condition : value }        // flips the condition
```

Displays value only when `condition` is true (or false with `?!`).
The result should either be a boolean value fetched from the world
or an [`Entity`](bevy::ecs::entity::Entity) with a [`FetchedCondition`](crate::FetchedCondition) component.

## Markdown

A subset of markdown features are supported:

* `*emphasis*`
* `**strong**`
* `__underline__`
* `~~strikethrough~~`
* `\*` escape character

## Whitespace Rule

Consecutive whitespaces are rendered either as one whitespace or multiple linebreaks.

## Inputs

* [`ParseValueFn`]
  * [`Text3dSegment::String`] should be returned for static values.
  * [`Text3dSegment::Extract`] should be returned after spawning a [`FetchedText`](crate::FetchedText) for dynamic values.
  * Since an index is provided, it is possible to return an empty segment and manually update it.
* [`ParseStyleFn`]
  Parses strings as [`SegmentStyle`].
* [`ParseConditionFn`]
  * [`ConditionOutput::Constant`] should be returned for static condition.
  * [`ConditionOutput::Dynamic`] should be returned after spawning a [`FetchedCondition`](crate::FetchedCondition) for dynamic values.

We trim whitespaces before passing arguments to these functions.
