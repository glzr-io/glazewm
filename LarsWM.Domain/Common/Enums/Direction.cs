using System;

namespace LarsWM.Domain.Common.Enums
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
      switch (direction)
      {
        case Direction.UP:
          return Direction.DOWN;
        case Direction.DOWN:
          return Direction.UP;
        case Direction.LEFT:
          return Direction.RIGHT;
        case Direction.RIGHT:
          return Direction.LEFT;
        default: throw new ArgumentOutOfRangeException();
      }
    }

    /// <summary>
    /// Get the layout that is needed for when moving or switching focus in given direction (eg. a
    /// horizontal layout when moving horizontally).
    /// </summary>
    public static Layout GetCorrespondingLayout(this Direction direction)
    {
      return (direction == Direction.LEFT || direction == Direction.RIGHT)
        ? Layout.HORIZONTAL : Layout.VERTICAL;
    }
  }
}
