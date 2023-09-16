using System;
using System.Reactive.Linq;
using System.Windows.Forms;
using GlazeWM.Infrastructure.WindowsApi;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Windows
{
  public partial class Inspector : Form
  {
    public IDisposable EventListener { get; }

    public Inspector()
    {
      InitializeComponent();
      MaximizeBox = false;
      MinimizeBox = false;
      FormBorderStyle = FormBorderStyle.FixedSingle;
      StartPosition = FormStartPosition.CenterParent;

      var point = new Point();
      EventListener = MouseEvents.MouseMoves
        .Sample(TimeSpan.FromMilliseconds(50))
        .Subscribe((@event) =>
        {
          if (point.X == @event.Point.X && point.Y == @event.Point.Y)
          {
            return;
          }

          point.X = @event.Point.X;
          point.Y = @event.Point.Y;
          var handle = WindowFromPoint(point);

          // get handle details
          var processName = WindowService.GetProcessOfHandle(handle)?.ProcessName ?? string.Empty;
          if (processName == "GlazeWM")
          {
            return;
          }

          var title = WindowService.GetTitleOfHandle(handle) ?? string.Empty;
          var className = WindowService.GetClassNameOfHandle(handle) ?? string.Empty;

          // update the inspector info
          titleValue.Text = title;
          classNameValue.Text = className;
          processNameValue.Text = processName;
        });
    }
  }
}
