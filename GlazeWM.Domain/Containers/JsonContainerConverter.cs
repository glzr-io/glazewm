using System;
using System.Linq;
using System.Text.Json;
using System.Text.Json.Serialization;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Serialization;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.Containers
{
  public class JsonContainerConverter : JsonConverter<Container>
  {
    public override bool CanConvert(Type typeToConvert)
    {
      return typeof(Container).IsAssignableFrom(typeToConvert);
    }

    public override Container Read(
      ref Utf8JsonReader reader,
      Type typeToConvert,
      JsonSerializerOptions options)
    {
      throw new NotSupportedException(
        $"Deserializing {typeToConvert.Name} from JSON is not supported."
      );
    }

    public override void Write(
      Utf8JsonWriter writer,
      Container value,
      JsonSerializerOptions options)
    {
      writer.WriteStartObject();

      // The following properties are required for all container types.
      WriteCommonProperties(writer, value, options);

      switch (value)
      {
        case Monitor monitor:
          WriteMonitorProperties(writer, monitor, options);
          break;
        case Workspace workspace:
          WriteWorkspaceProperties(writer, workspace, options);
          break;
        case SplitContainer splitContainer:
          WriteSplitContainerProperties(writer, splitContainer, options);
          break;
        case MinimizedWindow minimizedWindow:
          WriteWindowProperties(writer, minimizedWindow, options);
          WriteMinimizedWindowProperties(writer, minimizedWindow, options);
          break;
        case FloatingWindow floatingWindow:
          WriteWindowProperties(writer, floatingWindow, options);
          break;
        case TilingWindow tilingWindow:
          WriteWindowProperties(writer, tilingWindow, options);
          WriteTilingWindowProperties(writer, tilingWindow, options);
          break;
        default:
          break;
      }

      writer.WriteEndObject();
    }

    private static void WriteCommonProperties(
      Utf8JsonWriter writer,
      Container value,
      JsonSerializerOptions options)
    {
      writer.WriteNumber("X", value.X);
      writer.WriteNumber("Y", value.Y);
      writer.WriteNumber("Width", value.Width);
      writer.WriteNumber("Height", value.Height);
      writer.WriteNumber("FocusIndex", value.FocusIndex);
      writer.WriteString("Type", value.Type);

      // Recursively serialize child containers.
      writer.WriteStartArray("Children");
      foreach (var child in value.Children)
        Write(writer, child, options);

      writer.WriteEndArray();
    }

    private static void WriteMonitorProperties(
      Utf8JsonWriter writer,
      Monitor monitor,
      JsonSerializerOptions options)
    {
      writer.WriteString("DeviceName", monitor.DeviceName);
    }

    private static void WriteWorkspaceProperties(
      Utf8JsonWriter writer,
      Workspace workspace,
      JsonSerializerOptions options)
    {
      writer.WriteString("Name", workspace.Name);
    }

    private static void WriteSplitContainerProperties(
      Utf8JsonWriter writer,
      SplitContainer splitContainer,
      JsonSerializerOptions options)
    {
      writer.WriteString("Layout", splitContainer.Layout.ToString());
      writer.WriteNumber("SizePercentage", splitContainer.SizePercentage);
    }

    private static void WriteWindowProperties(
      Utf8JsonWriter writer,
      Window window,
      JsonSerializerOptions options)
    {
      writer.WriteNumber("Handle", tilingWindow.Handle.ToInt64());
      writer.WritePropertyName("FloatingPlacement");
      JsonSerializer.Serialize(writer, tilingWindow.FloatingPlacement);
      writer.WritePropertyName("BorderDelta");
      JsonSerializer.Serialize(writer, tilingWindow.BorderDelta);
    }

    private static void WriteMinimizedWindowProperties(
      Utf8JsonWriter writer,
      MinimizedWindow minimizedWindow,
      JsonSerializerOptions options)
    {
      writer.WriteString("PreviousState", minimizedWindow.PreviousState.ToString());
    }

    private static void WriteTilingWindowProperties(
      Utf8JsonWriter writer,
      TilingWindow tilingWindow,
      JsonSerializerOptions options)
    {
      writer.WriteNumber("SizePercentage", tilingWindow.SizePercentage);
    }
  }
}
