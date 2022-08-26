using System;
using System.Linq;
using System.Text.Json;
using System.Text.Json.Serialization;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.Containers
{
  public class ContainerConverter : JsonConverter<Container>
  {
    public override Container Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
    {
      using var jsonDocument = JsonDocument.ParseValue(ref reader);

      // Get the type of container (eg. "Workspace", "MinimizedWindow").
      var typeDiscriminator = jsonDocument.RootElement.GetProperty("__type").ToString();

      return DeserializeContainerJson(jsonDocument.RootElement, typeDiscriminator);
    }

    // TODO: Alternate name `CreateContainerFromType`.
    private static Container DeserializeContainerJson(
      JsonElement jsonObject,
      string typeDiscriminator,
      Container parent = null
    )
    {
      var newContainer = typeDiscriminator switch
      {
        "Container" => new Container(),
        "Monitor" => new Monitor(
          deviceName: jsonObject.GetProperty("DeviceName").GetString(),
          width: jsonObject.GetProperty("Width").GetInt32(),
          height: jsonObject.GetProperty("Height").GetInt32(),
          x: jsonObject.GetProperty("X").GetInt32(),
          y: jsonObject.GetProperty("Y").GetInt32()
        ),
        "Workspace" => new Workspace(
          jsonObject.GetProperty("Name").GetString()
        ),
        "SplitContainer" => new SplitContainer
        {
          Layout = jsonObject.GetProperty("Layout").GetString() == "HORIZONTAL" ? Layout.HORIZONTAL : Layout.VERTICAL,
          SizePercentage = jsonObject.GetProperty("SizePercentage").GetDouble()
        },
        "MinimizedWindow" => new MinimizedWindow(
          (IntPtr)jsonObject.GetProperty("Hwnd").GetInt64(),
          DeserializeFloatingPlacementJson(jsonObject.GetProperty("FloatingPlacement")),
          DeserializeBorderDeltaJson(jsonObject.GetProperty("BorderDelta")),
          jsonObject.GetProperty("PreviousState").GetString() == "TILING" ? WindowType.TILING : WindowType.FLOATING
        ),
        "FloatingWindow" => new FloatingWindow(
          (IntPtr)jsonObject.GetProperty("Hwnd").GetInt64(),
          DeserializeFloatingPlacementJson(jsonObject.GetProperty("FloatingPlacement")),
          DeserializeBorderDeltaJson(jsonObject.GetProperty("BorderDelta"))
        ),
        "TilingWindow" => new TilingWindow(
          // TODO: Handle `IntPtr` for 32-bit processes.
          (IntPtr)jsonObject.GetProperty("Hwnd").GetInt64(),
          DeserializeFloatingPlacementJson(jsonObject.GetProperty("FloatingPlacement")),
          DeserializeBorderDeltaJson(jsonObject.GetProperty("BorderDelta")),
          jsonObject.GetProperty("SizePercentage").GetDouble()
        ),
        _ => throw new ArgumentException(null, nameof(jsonObject)),
      };

      newContainer.Parent = parent;

      // TODO: Handle `ChildFocusOrder` based on `FocusIndex`.
      var children = jsonObject.GetProperty("Children").EnumerateArray();
      newContainer.Children = children
        .Select((child) =>
          DeserializeContainerJson(
            child,
            child.GetProperty("__type").GetString(),
            newContainer
          )
        )
        .ToList();

      return newContainer;
    }

    private static WindowRect DeserializeFloatingPlacementJson(JsonElement jsonObject)
    {
      var left = jsonObject.GetProperty("Left").GetInt32();
      var top = jsonObject.GetProperty("Top").GetInt32();
      var right = jsonObject.GetProperty("Right").GetInt32();
      var bottom = jsonObject.GetProperty("Bottom").GetInt32();
      return WindowRect.FromLTRB(left, top, right, bottom);
    }

    private static RectDelta DeserializeBorderDeltaJson(JsonElement jsonObject)
    {
      return null;
    }

    public override void Write(Utf8JsonWriter writer, Container value, JsonSerializerOptions options)
    {
      writer.WriteStartObject();
      writer.WriteNumber("X", value.X);
      writer.WriteNumber("Y", value.Y);
      writer.WriteNumber("Width", value.Width);
      writer.WriteNumber("Height", value.Height);
      writer.WriteString("__type", value.GetType().Name);

      // Handle focus index for root container.
      var focusIndex = value.Parent is not null ? value.FocusIndex : 0;
      writer.WriteNumber("FocusIndex", focusIndex);

      // Recursively serialize child containers.
      writer.WriteStartArray("Children");
      foreach (var child in value.Children)
        Write(writer, child, options);

      writer.WriteEndArray();
      writer.WriteEndObject();
    }
  }
}
