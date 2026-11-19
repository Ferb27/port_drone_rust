export type EventCallback = (payload: any) => void;

export class EventEmitter {
  private listeners: Record<string, EventCallback[]> = {};

  /**
   * Đăng ký lắng nghe một sự kiện
   * @param event Tên sự kiện (ví dụ: 'telemetry:sim_drone_1')
   * @param callback Hàm xử lý
   */
  public on(event: string, callback: EventCallback) {
    if (!this.listeners[event]) {
      this.listeners[event] = [];
    }
    this.listeners[event].push(callback);
  }

  /**
   * Hủy đăng ký lắng nghe sự kiện
   */
  public off(event: string, callback: EventCallback) {
    if (!this.listeners[event]) return;
    this.listeners[event] = this.listeners[event].filter(cb => cb !== callback);
  }

  /**
   * Phát ra một sự kiện
   * @param event Tên sự kiện
   * @param payload Dữ liệu đính kèm
   */
  public emit(event: string, payload: any) {
    if (!this.listeners[event]) return;
    this.listeners[event].forEach(callback => {
      try {
        callback(payload);
      } catch (error) {
        console.error(`Error in event listener for ${event}:`, error);
      }
    });
  }
}
