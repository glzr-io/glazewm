using System.Text.Json.Serialization;

namespace GlazeWM.Domain.Windows
{
  public class SerializeableWindow
  {
    public long Handle { get; set; }
    public string Title { get; }

    [JsonConstructor]
    public SerializeableWindow(long handle, string title)
    {
      Handle = handle;
      Title = title;
    }
  }
}
