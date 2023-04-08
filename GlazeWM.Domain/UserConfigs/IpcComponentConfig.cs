using System;

namespace GlazeWM.Domain.UserConfigs;

/// <summary>
/// Represents a label which is updated via IPC.
/// </summary>
public class IpcComponentConfig : BarComponentConfig, ICloneable
{
  /// <summary>
  /// Unique identifier for this label.
  /// </summary>
  public string LabelId { get; set; }

  /// <summary>
  /// Default text for this label.
  /// </summary>
  public string DefaultText { get; set; }

  /// <inheritdoc />
  public object Clone() => MemberwiseClone();
}
