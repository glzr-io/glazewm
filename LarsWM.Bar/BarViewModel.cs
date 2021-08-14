using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.ComponentModel;
using LarsWM.Domain.Workspaces;

namespace LarsWM.Bar
{
  public class BarViewModel : INotifyPropertyChanged
  {
    public event PropertyChangedEventHandler PropertyChanged;
    private ObservableCollection<Workspace> _workspaces = new ObservableCollection<Workspace>();
    public ObservableCollection<Workspace> Workspaces
    {
      get { return _workspaces; }
      set
      {
        _workspaces = value;
        OnPropertyChanged("Workspaces");
      }
    }

    public void AddWorkspace(Workspace workspace)
    {
      this.Workspaces.Add(workspace);
      OnPropertyChanged("Workspaces");
    }

    public void ClearWorkspaces()
    {
      this.Workspaces.Clear();
      OnPropertyChanged("Workspaces");
    }

    public void SetWorkspaces(List<Workspace> workspaces)
    {
      workspaces.Clear();

      foreach (var workspace in workspaces)
        workspaces.Add(workspace);

      OnPropertyChanged("Workspaces");
    }

    private void OnPropertyChanged(string propertyName)
    {
      PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
    }
  }
}
