using System;
using System.Globalization;
using System.Linq;
using System.Text.RegularExpressions;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Containers;
using GlazeWM.Infrastructure.Exceptions;

namespace GlazeWM.Domain.Windows
{
  public class ResizeParsingService
  {
    public static double ParseResizePercentage(
      Container containerToResize,
      ResizeDimension dimensionToResize,
      string resizeAmount)
    {
      try
      {
        var matchedResizeAmount = new Regex("(.*)(%|ppt|px)").Match(resizeAmount);
        var amount = matchedResizeAmount.Groups[1].Value;
        var unit = matchedResizeAmount.Groups[2].Value;
        var floatAmount = Convert.ToDouble(amount, CultureInfo.InvariantCulture);

        return unit switch
        {
          "%" => floatAmount / 100,
          "ppt" => floatAmount / 100,
          "px" => floatAmount * GetPixelScaleFactor(containerToResize, dimensionToResize),
          _ => throw new ArgumentException(null, nameof(resizeAmount)),
        };
      }
      catch
      {
        throw new FatalUserException($"Invalid resize amount {resizeAmount}.");
      }
    }

    private static double GetPixelScaleFactor(
      Container containerToResize,
      ResizeDimension dimensionToResize)
    {
      // Get available width/height that can be resized (ie. exclude inner gaps).
      var resizableLength = containerToResize.SelfAndSiblingsOfType<IResizable>().Aggregate(
        1.0,
        (sum, container) =>
          dimensionToResize == ResizeDimension.Width
            ? sum + container.Width
            : sum + container.Height
      );

      return 1.0 / resizableLength;
    }
  }
}
