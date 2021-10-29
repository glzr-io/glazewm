using System.Collections.Generic;
using System.Linq;

namespace GlazeWM.Infrastructure.Utils
{
  public static class EnumerableExtensions
  {
    /// <summary>
    /// Create an enumerable of tuples with index and item at index. Can be used with foreach loops.
    /// </summary>
    public static IEnumerable<(T item, int index)> WithIndex<T>(this IEnumerable<T> source)
    {
      return source?.Select((item, index) => (item, index)) ?? new List<(T, int)>();
    }
  }
}
