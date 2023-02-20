using System;
using System.ComponentModel;

namespace GlazeWM.Bar.Common
{
  public class ViewModelBase : INotifyPropertyChanged, IDisposable
  {
    public event PropertyChangedEventHandler PropertyChanged;

    public void Dispose()
    {
      Dispose(true);
      GC.SuppressFinalize(this);
    }

    protected virtual void Dispose(bool disposing)
    {
    }

    protected void OnPropertyChanged(string propertyName)
    {
      PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
    }
  }
}
