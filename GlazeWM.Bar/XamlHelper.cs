using System;
using System.Collections.Generic;
using System.Linq;
using System.Xml.Linq;
using GlazeWM.Bar.Components;
using GlazeWM.Infrastructure.Utils;

namespace GlazeWM.Bar
{
  public static class XamlHelper
  {
    /// <summary>
    /// Convert size properties from user config (eg. `FontSize`) to be XAML
    /// compatible.
    /// </summary>
    public static string FormatSize(string size)
    {
      return $"{UnitsHelper.TrimUnits(size)}";
    }

    /// <summary>
    /// Convert color properties from user config (eg. `Background`) to be XAML
    /// compatible. Colors in the user config are specified in RGBA, whereas XAML expects
    /// ARGB.
    /// </summary>
    public static string FormatColor(string color)
    {
      var isHexColor = color.StartsWith("#", StringComparison.InvariantCulture);

      if (!isHexColor)
        return color;

      var rgbaHex = color.Replace("#", "");
      var argbHex = rgbaHex.Length == 8 ? rgbaHex[6..8] + rgbaHex[0..6] : rgbaHex;

      return $"#{argbHex}";
    }

    /// <summary>
    /// Convert shorthand properties from user config (ie. `Padding`, `Margin`, and
    /// `BorderWidth`) to be compatible with their equivalent XAML properties (ie.
    /// `Padding`, `Margin`, and `BorderThickness`). Shorthand properties follow the
    /// 1-to-4 value syntax used in CSS.
    /// </summary>
    /// <exception cref="ArgumentException"></exception>
    public static string FormatRectShorthand(string shorthand)
    {
      var shorthandParts = shorthand.Split(" ")
        .Select(shorthandPart => UnitsHelper.TrimUnits(shorthandPart))
        .ToList();

      return shorthandParts.Count switch
      {
        1 => shorthand,
        2 => $"{shorthandParts[1]},{shorthandParts[0]},{shorthandParts[1]},{shorthandParts[0]}",
        3 => $"{shorthandParts[1]},{shorthandParts[0]},{shorthandParts[1]},{shorthandParts[2]}",
        4 => $"{shorthandParts[3]},{shorthandParts[0]},{shorthandParts[1]},{shorthandParts[2]}",
        _ => throw new ArgumentException(null, nameof(shorthand)),
      };
    }

    public static LabelViewModel ParseLabel(
      string labelString,
      Dictionary<string, string> labelVariables)
    {
      var labelWithVariables = labelString;

      // Replace variables in label with their corresponding variable.
      foreach (var (key, value) in labelVariables)
        labelWithVariables = labelWithVariables.Replace($"{{{key}}}", value);

      // Wrap `labelString` in arbitrary tag to make it valid XML.
      var wrappedLabel = $"<Label>{labelString}</Label>";
      var labelXml = XElement.Parse(wrappedLabel);

      var labelNode = labelXml.FirstNode;
      var labelSpans = new List<LabelSpan>();

      while (labelNode is not null)
      {
        var labelSpan = new LabelSpan(labelNode.ToString());
        labelSpans.Add(labelSpan);
        labelNode = labelNode.NextNode;
      }

      return new LabelViewModel(labelSpans);
    }
  }
}
