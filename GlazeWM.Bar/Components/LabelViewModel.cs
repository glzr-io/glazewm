using System.Collections.Generic;
using GlazeWM.Bar.Common;

namespace GlazeWM.Bar.Components
{
  public class LabelViewModel : ViewModelBase
  {
    public List<LabelSpan> Spans { get; }

    public LabelViewModel(List<LabelSpan> spans)
    {
      Spans = spans;
    }
  }
}
