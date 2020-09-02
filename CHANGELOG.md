# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.3.1 (OrbTk next) wip

### 0.3.1-alpha4 (wip)

* PasswordBox widget
* Clipboard service
* TextBehavior: Copy Ctrl+C, Paste Ctrl+V, Cut Ctrl+X
* orbraq backend (Orbclient and raqote)
* Refactor on_changed callback, add key parameter
* Access RawWindowHandle from Context
* Select colors in themes through of CSS-like functions
* Create `Color` from HSV and HSL values
* Create a `Color` by its CSS name
* Gradient coordinates become relative to the path
* Text mark with Shift + Left | Shift + Right

### 0.3.1-alpha3

* Dynamic theme switch
* Add all material font icons as resource
* Replaces css-engine with custom Rust/Ron based theming
* Add widget access helpers for states
* API update check deprecated methods an replace to new ones
* Performance improvements
* Change state update order from tree order to incoming changes order
* NumericBox widget
* Update caret position on TextBox by mouse click
* Text input support for ', /, \, [, ], {, }
* Multiple window support (experimental)
* Pathfinder / Glutin backend (experimental)
* ProgressBar widget
* Measure distance between two Points
* Improve: Mouse event arguments
* Fix: Crash when a child widget is removed
* TabWidget widget
* Add on_changed property change callback to all widgets
* OrbTk book (manual) wip

### 0.3.1-alpha2

* ComboBox / ComboboxItem widget
* Slider widget
* Popup widget
* Overlay layer
* Service registry for states
* Settings service (serialize / deserialize data)
* Direct access of states in callbacks
* Impl RawWindowHandle for Context (wip)
* Sent requests to window shell
* Layout fixes and stack layout example
* Many web fixes
* State cleanup method
* Refactor setting of styling selectors
* TextBox select all (Ctrl + a)
* Text input support for !, @, #
* Borderless window

### 0.3.1-alpha1

* api crate: base api elements of OrbTk e.g. widget and application parts
* css-engine crate: parse and read values from a css file
* proc_macros crate: procedural helper macros
* render crate: cross platform 2D/3D render library
* shell crate: cross platform window and event handling
* theme crate: OrbTks default theme (light and dark)
* tree crate: tree structure based on DCES
* utils crate: helper structs and traits
* widgets crate: base widget library
* Button widget
* Canvas widget
* CheckBox widget
* Container widget
* Cursor widget
* FontIconBlock widget
* Image widget
* Items widget
* ListView widget
* ScrollBar widget
* ScrollIndicator widget
* ScrollViewer widget
* Stack widget
* Switch widget
* TextBlock widget
* TextBox widget
* ToggleButton widget
* Window widget
