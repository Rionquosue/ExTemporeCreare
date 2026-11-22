## File structure rules

The full rules, tool names and script locations are in `filestructure.md`. In short:

1. Code reused by several scripts goes into a class. Clasess should be stored in a subfolder and documented correctly.
2. Keep UI and feature logic in separate scripts and types. Follow the UI placement and naming rules in `filestructure.md`.
5. Tools are to get their own folder in a codebase. Each tool should have all of its functional code contained in that folder. UI elements are to be in a subfolder of the UI folder defined in `filestructure.md`
6. One job per script. Do not create a monolith.

## Factoring and name rules

1. All names must be consistent. If a tool is named "FlipTool" then all classes and scripts related should be along the lines of "FlipToolMenu.kt" or "FlipToolChecks.kt"
2. All files must be human readable and refactorable. Keep names descriptive, comment where clarity is needed. An example is commenting "This segment calls components/FuzzyMatch.kt to embed a fuzzy search window."
2. All names must be simple and descriptive. Things should be named something like "PreFlight" if they run before, "PostFlight" if after. 

## UI Design rules

1. Always divide things cleanly, do not overload the user with a lot of information all at once.
2. If a tool requires more context than can be cleanly given in a small box, or interaction would suffer from the size, create a dropdown, expandable window, or popup depending on requirements.
3. User agency is of utmost priority. The most impactful tools such as visual filters or destructive tools should always be presented up front.
4. In the case of a destructive tool, always provide a warning. Prompt the user to save before running a destructive tool.
5. Avoid redundant UI elements, if something is defined in a parent window, only present what is required for user context in any expanding menus. 

## Undo and consistency rules

1. All actions that can be applied in a single click should be undone in a single ctrl-z step.
2. Always check for overlapping UI elements that would degrade user experience.
3. Always check for any bugs, race conditions, or interrupts that may destroy user data unintentionally.
