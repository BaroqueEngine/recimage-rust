use image::*;

#[derive(Debug)]
struct Rect {
  left: u32,
  right: u32,
  top: u32,
  bottom: u32,
}

#[derive(Debug)]
struct Quad {
  rect: Rect,
  score: f32,
  color: Rgba<u8>,
}

#[derive(Debug)]
struct CalcAndColor {
  color: u8,
  error: f32,
}

static AREA_POWER: f32 = 0.2;
static MIN_SIZE: u32 = 4;
static INF: f32 = 9999.0;

impl Rect {
  fn split(&self) -> [Rect; 4] {
    let center_x = (self.left + self.right) / 2;
    let center_y = (self.top + self.bottom) / 2;

    [
        Rect {
          left: self.left,
          right: center_x - 1,
          top: self.top,
          bottom: center_y - 1,
        },
        Rect {
          left: center_x,
          right: self.right,
          top: self.top,
          bottom: center_y - 1,
        },
        Rect {
          left: self.left,
          right: center_x - 1,
          top: center_y,
          bottom: self.bottom,
        },
        Rect {
          left: center_x,
          right: self.right,
          top: center_y,
          bottom: self.bottom,
        },
    ]
  }
}

fn get_rect_color(img: &DynamicImage, rect: &Rect, color_index: usize) -> [i32; 256] {
  let mut ret: [i32; 256] = [0; 256];

  for y in rect.top..=rect.bottom {
    for x in rect.left..=rect.right {
      let pixel = img.get_pixel(x, y);
      let col = pixel[color_index];
      ret[col as usize] += 1;
    }
  }

  ret
}

fn calc_color_and_error(img: &DynamicImage, rect: &Rect, color_index: usize) -> CalcAndColor {
  let hist = get_rect_color(&img, &rect, color_index);
  let total = hist.iter().enumerate().fold(0, |sum, (i, v)| sum + (i as i32) * v);
  let num = hist.iter().fold(0, |sum, v| sum + v);
  let avg: f32 = (total as f32) / (num as f32);
  let error = ((hist.iter().enumerate().fold(0.0, |sum, (i, v)| (sum as f32) + ((*v as f32) * ((i as f32) - avg).powf(2.0))) as f32) / (num as f32)).sqrt();

  CalcAndColor { color: avg as u8, error }
}

fn calc_area(rect: &Rect) -> u32 {
  (rect.right - rect.left + 1) * (rect.bottom - rect.top + 1)
}

fn create_quad(img: &DynamicImage, rect: Rect) -> Quad {
  let r = calc_color_and_error(img, &rect, 0);
  let g = calc_color_and_error(img, &rect, 1);
  let b = calc_color_and_error(img, &rect, 2);
  let color = Rgba([r.color, g.color, b.color, 255]);
  let error = r.error * 0.2989 + g.error * 0.587 + b.error * 0.114;
  let mut score = error * (calc_area(&rect) as f32).powf(AREA_POWER);

  if rect.right - rect.left + 1 <= MIN_SIZE || rect.bottom - rect.top + 1 <= MIN_SIZE {
    score = -INF;
  }

  return Quad { rect, score, color };
}

fn render(img: &mut DynamicImage, quad: &Quad) {
  let rect = &quad.rect;
  for y in rect.top..=rect.bottom {
    for x in rect.left..=rect.right {
      if x == rect.left || x == rect.right || y == rect.top || y == rect.bottom {
        img.put_pixel(x, y, Rgba([0, 0, 0, 255]));
      }
      else {
        img.put_pixel(x, y, quad.color);
      }
    }
  }
}

fn draw(src: &DynamicImage, dst: &mut DynamicImage, heap: &mut Vec<Quad>) {
  // todo: replace with BinaryHeap
  let mut best_i: usize = 0;
  for (i, _) in heap.iter().enumerate() {
    if heap[i].score > heap[best_i].score {
      best_i = i;
    }
  }
  let quad = heap.swap_remove(best_i);

  let rects = quad.rect.split();
  for rect in rects {
    let child = create_quad(&src, rect);
    render(dst, &child);
    heap.push(child);
  }
}

fn main() {
  let mut heap: Vec<Quad> = Vec::new();
  let src = image::open("input.png").unwrap();
  let (w, h) = src.dimensions();
  let mut dst = src.clone();
  let root = create_quad(&src, Rect { left: 0, right: w - 1, top: 0, bottom: h - 1 });
  render(&mut dst, &root);
  heap.push(root);

  for i in 0..500 {
    draw(&src, &mut dst, &mut heap);
    // dst.save(format!("dest/{}.png", i)).unwrap();
  }

  dst.save(format!("{}.png", "output")).unwrap();
}