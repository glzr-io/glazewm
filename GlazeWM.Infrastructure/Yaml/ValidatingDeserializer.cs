using System;
using System.ComponentModel.DataAnnotations;
using YamlDotNet.Core;
using YamlDotNet.Serialization;

namespace GlazeWM.Infrastructure.Yaml
{
  public class ValidatingDeserializer : INodeDeserializer
  {
    private readonly INodeDeserializer _nodeDeserializer;

    public ValidatingDeserializer(INodeDeserializer nodeDeserializer)
    {
      _nodeDeserializer = nodeDeserializer;
    }

    public bool Deserialize(IParser parser, Type expectedType,
        Func<IParser, Type, object> nestedObjectDeserializer, out object value)
    {
      if (!_nodeDeserializer.Deserialize(parser, expectedType, nestedObjectDeserializer, out value))
        return false;

      // Validate using the provided data annotations (eg. `[Required]`).
      var context = new ValidationContext(value, null, null);
      Validator.ValidateObject(value, context, true);

      return true;
    }
  }
}
