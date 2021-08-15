using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Windows.Threading;
using LarsWM.Domain.Workspaces;

namespace LarsWM.Bar
{
  public class BarViewModel : INotifyPropertyChanged
  {
    public event PropertyChangedEventHandler PropertyChanged;
    private readonly Dispatcher _dispatcher;
    private ObservableCollection<Workspace> _workspaces = new ObservableCollection<Workspace>();
    public ObservableCollection<Workspace> Workspaces
    {
      get { return this._workspaces; }
      set
      {
        this._workspaces = value;
        this.OnPropertyChanged("Workspaces");
      }
    }

    public BarViewModel(Dispatcher dispatcher)
    {
      this._dispatcher = dispatcher;
    }

    public void SetWorkspaces(IEnumerable<Workspace> workspaces)
    {
      this._dispatcher.BeginInvoke(new Action(() =>
     {
       var newWorkspaces = new ObservableCollection<Workspace>();

       foreach (var workspace in workspaces)
         newWorkspaces.Add(workspace);

       this.Workspaces = newWorkspaces;
     }));
    }

    private void OnPropertyChanged(string propertyName)
    {
      this.PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
    }
  }
}
