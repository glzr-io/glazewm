using System;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace GlazeWM.Domain.Containers
{
  public class JsonContainerConverterFactory : JsonConverterFactory
  {
    public override bool CanConvert(Type typeToConvert)
    {
      throw new NotImplementedException();
    }

    public override JsonConverter CreateConverter(
      Type typeToConvert,
      JsonSerializerOptions options)
    {
      throw new NotImplementedException();
    }
  }
}
