using System.IO;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;
using YamlDotNet.Serialization.NodeDeserializers;

namespace GlazeWM.Infrastructure.Yaml
{
  public class YamlDeserializationService
  {
    IDeserializer _deserializer = new DeserializerBuilder()
      .WithNamingConvention(PascalCaseNamingConvention.Instance)
      .WithNodeDeserializer(
        inner => new ValidatingDeserializer(inner),
        component => component.InsteadOf<ObjectNodeDeserializer>()
      )
      .Build();

    public T Deserialize<T>(TextReader input)
    {
      return _deserializer.Deserialize<T>(input);
    }
  }
}

