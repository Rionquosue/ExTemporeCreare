# ExTemporeCreare
A mobile android app designed to take the best parts of several existing ones and make something a little less clunky.

## For AI agents and coding tools

Before making any change, read [docs/for_agents/](docs/for_agents/), starting with [for_agents.md](docs/for_agents/for_agents.md). It sets instruction priority, reporting format, commit rules and code conventions. Related files in that folder:

- [script_structure.md](docs/for_agents/script_structure.md): file, naming, UI and undo rules for scripts.
- [bugcheck.md](docs/for_agents/bugcheck.md): bugcheck and resolution procedures before a mainline release.

Use [docs/filestructure.md](docs/filestructure.md) to find documents and code without searching the whole repository.

## Technology and standards

- **Platform:** Android, tablet-first, with adaptations for smaller screens like phones.
- **UI and app layer:** Kotlin with Jetpack Compose.
- **Engine:** Rust, in [engine/](engine/).
- **Build:** Gradle (Android Studio project).
- **Native file format:** `.etc`, an archive format derived from OpenRaster (ORA), with `.etcr` as a separate reference-board file. See [filestandard.md](docs/Standardsdocs/filestandard.md).
- **Application behaviour:** startup, session resume, infinite canvas and export regions. See [applicationstandard.md](docs/Standardsdocs/applicationstandard.md).
- **UI specification:** [uistandards.md](docs/Standardsdocs/UIStandards/uistandards.md), with the interactive mockup [Ex-Tempore-Creare_UI_Mockup_v10.html](docs/Ex-Tempore-Creare_UI_Mockup_v10.html) as its reference implementation.
- **Tool and filter research:** [graphic_design_tools_and_filters.md](docs/Standardsdocs/Toolstandards/graphic_design_tools_and_filters.md).
- **License:** GPL-3.0, see [LICENSE](LICENSE).

## Repository layout

- [engine/](engine/): Rust background engine.
- [interface/](interface/): user interface code.
- [tools/](tools/): tool code, one folder per tool.
- [components/](components/): code shared by several tools or UI elements.
- [dependencies/](dependencies/): dependencies required to build.
- [docs/](docs/): standards, agent instructions and indexe, active documentation, general how-to and tool documentation drafts for 'help' documentation. ([classes.md](docs/classes.md), [variables.md](docs/variables.md)).
