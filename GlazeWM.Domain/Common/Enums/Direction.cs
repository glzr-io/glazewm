using System;

namespace GlazeWM.Domain.Common.Enums
{
  public enum Direction
  {
    UP,
    DOWN,
    LEFT,
    RIGHT,
  }

  public static class DirectionExtensions
  {
    /// <summary>
    /// Get the inverse of a given direction (eg. `Direction.UP` is the inverse of `Direction.DOWN`).
    /// </summary>
    public static Direction Inverse(this Direction direction)
    {
      return direction switch
      {
        Direction.UP => Direction.DOWN,
        Direction.DOWN => Direction.UP,
        Direction.LEFT => Direction.RIGHT,
        Direction.RIGHT => Direction.LEFT,
        _ => throw new ArgumentOutOfRangeException(nameof(direction)),
      };
    }

    /// <summary>
    /// Get the layout that is needed for when moving or switching focus in given direction (eg. a
    /// horizontal layout when moving horizontally).
    /// </summary>
    public static Layout GetCorrespondingLayout(this Direction direction)
    {
      return (direction is Direction.LEFT or Direction.RIGHT)
        ? Layout.HORIZONTAL : Layout.VERTICAL;
    }
  }
}
