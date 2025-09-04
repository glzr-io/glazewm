<context>
  <project_overview>
    <summary>
      GlazeWM is a window manager for Windows and macOS written in Rust.
    </summary>

    <crates>
      <crate name="wm" kind="bin">
        <purpose
          >Main application implementing core window management logic.</purpose
        >
        <install_path>C:\Program Files\glzr.io\glazewm.exe</install_path>
      </crate>

      <crate name="wm-cli" kind="bin,lib">
        <purpose>CLI for interacting with the main application.</purpose>
        <install_path>C:\Program Files\glzr.io\cli\glazewm.exe</install_path>
        <note>Added to $PATH by default.</note>
      </crate>

      <crate name="wm-common" kind="lib">
        <purpose
          >Shared types, utilities, and constants used across other
          crates.</purpose
        >
      </crate>

      <crate name="wm-ipc-client" kind="lib">
        <purpose
          >WebSocket client library for IPC with the main application.</purpose
        >
      </crate>

      <crate name="wm-platform" kind="lib">
        <purpose
          >Wrappers over platform-specific APIs; other crates do not call
          Windows/macOS APIs directly.</purpose
        >
      </crate>

      <crate name="wm-watcher" kind="bin">
        <purpose
          >Watchdog process ensuring proper cleanup when the main application
          exits.</purpose
        >
        <install_path
          >C:\Program Files\glzr.io\glazewm-watcher.exe</install_path
        >
      </crate>
    </crates>
  </project_overview>

  <guidelines>
    <code_style_and_formatting>
      <rule>Do not leave partial or simplified implementations.</rule>
      <rule>Avoid .unwrap() wherever possible.</rule>
      <rule
        >Follow clippy suggestions unless there is a compelling reason not
        to.</rule
      >
      <rule>Use rust-analyzer with clippy for continuous linting.</rule>
      <toolchain channel="nightly">
        <policy
          >Only use nightly features when they provide clear benefit.</policy
        >
      </toolchain>
    </code_style_and_formatting>

    <documentation>
      <rule
        >Document public APIs with rustdoc comments, especially for the
        wm-platform crate.</rule
      >

      <rustdoc_requirements>
        <item required="true">Concise summary of the function or type.</item>
        <item required="true">Notable caveats for usage (kept brief).</item>
        <item
          >Return value note if unclear (e.g., Returns a vector of NativeMonitor
          sorted left-to-right.).</item
        >
        <item heading="# Panics">List cases where the function can panic.</item>
        <item heading="# Platform-specific">Platform-specific notes.</item>
        <item heading="# Examples">Example usage (usually optional).</item>
      </rustdoc_requirements>

      <notes>
        <note
          >Use punctuation at the end of doc comments and in-line
          comments.</note
        >
        <note
          >Wrap type names in code style when referenced (e.g.,
          ExampleStruct).</note
        >
        <note>If using unsafe features, include a "SAFETY: ..." comment.</note>
      </notes>
    </documentation>

    <testing>
      <rule>Use #[cfg(test)] for test modules.</rule>
      <rule>Write unit tests for core functionality.</rule>
      <note
        >Tests verify correctness; provide a principled implementation that
        follows best practices.</note
      >
    </testing>

    <error_handling>
      <rule>Prefer thiserror over anyhow in the wm-platform crate.</rule>
      <rule
        >Use thiserror for custom error types with #[derive(Debug,
        thiserror::Error)].</rule
      >
    </error_handling>

    <logging_and_tracing>
      <rule>Use the tracing crate for logging.</rule>
      <levels>
        <level>error!</level>
        <level>warn!</level>
        <level>info!</level>
        <level>debug!</level>
      </levels>
    </logging_and_tracing>
  </guidelines>

  <other_context load_condition="if working with wm-platform crate">./thoughts/wm-platform-guide.md</other_context>

  <feasibility_note>
    If a task is unreasonable or infeasible, or if any tests are incorrect,
    state this explicitly. Solutions should be robust, maintainable, and
    extendable.
  </feasibility_note>
</context>
