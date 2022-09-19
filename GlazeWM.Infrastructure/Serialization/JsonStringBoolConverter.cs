using System;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace GlazeWM.Infrastructure.Serialization
{
  public class JsonStringBoolConverter : JsonConverter<bool>
  {
    /// <summary>
    /// Allow boolean strings when deserializing to a bool (eg. "true" -> true).
    /// </summary>
    /// <exception cref="JsonException"></exception>
    public override bool Read(
      ref Utf8JsonReader reader,
      Type typeToConvert,
      JsonSerializerOptions options)
    {
      return reader.TokenType switch
      {
        JsonTokenType.True => true,
        JsonTokenType.False => false,
        JsonTokenType.String => bool.TryParse(reader.GetString(), out var parsedBool)
          ? parsedBool
          : throw new JsonException(),
        _ => throw new JsonException(),
      };
    }

    public override void Write(Utf8JsonWriter writer, bool value, JsonSerializerOptions options)
    {
      writer.WriteBooleanValue(value);
    }
  }
}
