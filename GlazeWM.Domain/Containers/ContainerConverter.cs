using System;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace GlazeWM.Domain.Containers
{
  public class ContainerConverter : JsonConverter<Container>
  {
    public override Container Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
    {
      using var jsonObject = JsonDocument.ParseValue(ref reader);

      return null;
    }

    public override void Write(Utf8JsonWriter writer, Container value, JsonSerializerOptions options)
    {
      writer.WriteStartObject();
      writer.WriteNumber("X", value.X);
      writer.WriteNumber("Y", value.Y);
      writer.WriteNumber("Width", value.Width);
      writer.WriteNumber("Height", value.Height);
      writer.WriteString("Type", value.GetType().Name);

      writer.WriteStartArray("Children");

      foreach (var child in value.Children)
        Write(writer, child, options);

      writer.WriteEndArray();
      writer.WriteEndObject();
    }
  }
}
