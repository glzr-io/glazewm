using System;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace GlazeWM.Infrastructure.Serialization
{
  public class JsonIntPtrConverter : JsonConverter<IntPtr>
  {
    public override IntPtr Read(
      ref Utf8JsonReader reader,
      Type typeToConvert,
      JsonSerializerOptions options)
    {
      return new IntPtr(reader.GetInt64());
    }

    public override void Write(
      Utf8JsonWriter writer,
      IntPtr value,
      JsonSerializerOptions options)
    {
      writer.WriteNumberValue(value.ToInt64());
    }
  }
}
