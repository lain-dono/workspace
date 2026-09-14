var com_watabou_mfcg_Main = function () {
	com_watabou_utils_Random.reset();
	com_watabou_coogee_ui_UIStyle.useDefault();
	com_watabou_system_State.init(null, com_watabou_mfcg_Values.prepare);
	com_watabou_mfcg_mapping_Style.restore();
	this.stage.showDefaultContextMenu = false;
	com_watabou_system_URLState.baseURL = "https://watabou.github.io/city-generator/0.10.0";
	com_watabou_mfcg_Main.preview = com_watabou_system_URLState.getFlag("preview");
	var bp = com_watabou_mfcg_model_Blueprint.fromURL();
	if (bp == null) {
		bp = com_watabou_mfcg_model_Blueprint.create(25, com_watabou_utils_Random.seed);
	}
	new com_watabou_mfcg_model_City(bp);
	com_watabou_coogee_Game.call(this, com_watabou_mfcg_scenes_ViewScene);
};
$hxClasses["com.watabou.mfcg.Main"] = com_watabou_mfcg_Main;
com_watabou_mfcg_Main.__name__ = "com.watabou.mfcg.Main";
com_watabou_mfcg_Main.__super__ = com_watabou_coogee_Game;
com_watabou_mfcg_Main.prototype = $extend(com_watabou_coogee_Game.prototype, {
	getScale: function (w, h) {
		var fitZoom = Math.min(w / com_watabou_mfcg_Main.MIN_WIDTH, h / com_watabou_mfcg_Main.MIN_HEIGHT);
		var screenRes = openfl_system_Capabilities.get_screenDPI() / 72;
		return 1;
	}
	, switchSceneImp: function (scClass) {
		if (com_watabou_coogee_ui_UI.layer == null) {
			com_watabou_coogee_ui_UI.layer = new com_watabou_coogee_ui_View();
		}
		com_watabou_coogee_Game.prototype.switchSceneImp.call(this, scClass);
		this.addChild(com_watabou_coogee_ui_UI.layer);
	}
	, layout: function () {
		var w = this.stage.stageWidth;
		var h = this.stage.stageHeight;
		var scale = this.getScale(w, h);
		com_watabou_coogee_ui_UI.layer.set_scaleX(com_watabou_coogee_ui_UI.layer.set_scaleY(scale));
		com_watabou_coogee_ui_UI.layer.setSize(w / scale, h / scale);
		com_watabou_coogee_Game.prototype.layout.call(this);
	}
	, __class__: com_watabou_mfcg_Main
});
var DocumentClass = function (current) {
	current.addChild(this);
	com_watabou_mfcg_Main.call(this);
	this.dispatchEvent(new openfl_events_Event("addedToStage", false, false));
};
$hxClasses["DocumentClass"] = DocumentClass;
DocumentClass.__name__ = "DocumentClass";
DocumentClass.__super__ = com_watabou_mfcg_Main;
DocumentClass.prototype = $extend(com_watabou_mfcg_Main.prototype, {
	__class__: DocumentClass
});
var com_watabou_mfcg_Buffer = function () { };
$hxClasses["com.watabou.mfcg.Buffer"] = com_watabou_mfcg_Buffer;
com_watabou_mfcg_Buffer.__name__ = "com.watabou.mfcg.Buffer";
com_watabou_mfcg_Buffer.read = function (handler) {
	$global.navigator.clipboard.readText().then(handler);
};
com_watabou_mfcg_Buffer.write = function (txt) {
	$global.navigator.clipboard.writeText(txt);
};
var com_watabou_mfcg_Values = function () { };
$hxClasses["com.watabou.mfcg.Values"] = com_watabou_mfcg_Values;
com_watabou_mfcg_Values.__name__ = "com.watabou.mfcg.Values";
com_watabou_mfcg_Values.prepare = function (data) {
	data["random"] = true;
	data["citadel"] = true;
	data["urban_castle"] = false;
	data["walls"] = true;
	data["river"] = false;
	data["coast"] = true;
	data["temple"] = true;
	data["plaza"] = true;
	data["shantytown"] = false;
	data["farms"] = true;
	data["green"] = false;
	data["hub"] = false;
	data["gates"] = -1;
	data["display_mode"] = "Lots";
	data["lots_method"] = "Twisted";
	data["processing"] = "Offset";
	data["towers"] = "Round";
	data["landmarks"] = "Icon";
};
var com_watabou_mfcg_annotations_ArcLabelSlot = function (pos, angle, radius, span) {
	this.pos = pos;
	this.angle = angle;
	this.radius = radius;
	this.span = span;
};
$hxClasses["com.watabou.mfcg.annotations.ArcLabelSlot"] = com_watabou_mfcg_annotations_ArcLabelSlot;
com_watabou_mfcg_annotations_ArcLabelSlot.__name__ = "com.watabou.mfcg.annotations.ArcLabelSlot";
com_watabou_mfcg_annotations_ArcLabelSlot.prototype = {
	__class__: com_watabou_mfcg_annotations_ArcLabelSlot
};
var com_watabou_mfcg_annotations_Area = function (area) {
	this.area = area;
};
$hxClasses["com.watabou.mfcg.annotations.Area"] = com_watabou_mfcg_annotations_Area;
com_watabou_mfcg_annotations_Area.__name__ = "com.watabou.mfcg.annotations.Area";
com_watabou_mfcg_annotations_Area.fullSpan = function (points) {
	return [points[0], points[points.length - 1]];
};
com_watabou_mfcg_annotations_Area.largestSpan = function (points) {
	var largest = -1;
	var length = 0.0;
	var i = 0;
	while (i < points.length) {
		var len = openfl_geom_Point.distance(points[i], points[i + 1]);
		if (length < len) {
			length = len;
			largest = i;
		}
		i += 2;
	}
	return [points[largest], points[largest + 1]];
};
com_watabou_mfcg_annotations_Area.prototype = {
	arcLabel: function () {
		return this.measure(true);
	}
	, horLabel: function () {
		return this.measure(false);
	}
	, measure: function (arced) {
		var box = arced ? com_watabou_geom_polygons_PolyBounds.obb(this.area) : com_watabou_geom_polygons_PolyBounds.aabb(this.area);
		var short = box[0].subtract(box[3]);
		var long = box[1].subtract(box[0]);
		if (arced && Math.abs(long.y / long.x) > Math.pow(long.get_length() / short.get_length(), 2)) {
			short = box[1].subtract(box[0]);
			long = box[2].subtract(box[1]);
		}
		var centroid = com_watabou_geom_polygons_PolyCore.centroid(this.area);
		var vLimits = com_watabou_mfcg_annotations_Area.largestSpan(com_watabou_geom_polygons_PolyCut.pierce(this.area, centroid, centroid.add(short)));
		var v0 = vLimits[0];
		var v1 = vLimits[1];
		var bestSpan = null;
		var bestSpanSize = -Infinity;
		var _g = 0;
		var _g1 = com_watabou_mfcg_annotations_Area.marks;
		while (_g < _g1.length) {
			var ratio = _g1[_g];
			++_g;
			var c = com_watabou_geom_GeomUtils.lerp(v0, v1, ratio);
			var span = com_watabou_mfcg_annotations_Area.largestSpan(com_watabou_geom_polygons_PolyCut.pierce(this.area, c, c.add(long)));
			var spanSize = openfl_geom_Point.distance(span[0], span[1]);
			if (bestSpanSize < spanSize) {
				bestSpanSize = spanSize;
				bestSpan = span;
			}
		}
		var pos = com_watabou_geom_GeomUtils.lerp(bestSpan[0], bestSpan[1]);
		if (arced) {
			var angle = Math.atan(long.y / long.x);
			var t = 2 * com_watabou_utils_PointExtender.project(short, pos.subtract(centroid));
			t = t > 0 ? Math.sqrt(t) : -Math.sqrt(-t);
			var radius = short.get_length() / (2 * t);
			if (long.x > 0) {
				radius = -radius;
			}
			return new com_watabou_mfcg_annotations_ArcLabelSlot(pos, angle, radius, bestSpanSize);
		} else {
			return new com_watabou_mfcg_annotations_ArcLabelSlot(pos, 0, Infinity, bestSpanSize);
		}
	}
	, __class__: com_watabou_mfcg_annotations_Area
};
var com_watabou_mfcg_export_Export = function () { };
$hxClasses["com.watabou.mfcg.export.Export"] = com_watabou_mfcg_export_Export;
com_watabou_mfcg_export_Export.__name__ = "com.watabou.mfcg.export.Export";
com_watabou_mfcg_export_Export.asPNG = function () {
	var model = com_watabou_mfcg_model_City.instance;
	var realWidth = model.maxx - model.minx + 40.0;
	var realHeight = model.maxy - model.miny + 40.0;
	var mul = Math.sqrt(16777216 / (realWidth * realHeight));
	var pngWidth = mul * realWidth | 0;
	var pngHeight = mul * realHeight | 0;
	var bmp = new openfl_display_BitmapData(pngWidth, pngHeight, false, model.waterEdge.length > 0 ? com_watabou_mfcg_mapping_Style.colorWater : com_watabou_mfcg_mapping_Style.colorPaper);
	var scene = com_watabou_mfcg_scenes_TownScene.instance;
	var scWidth = scene.rWidth;
	var scHeight = scene.rHeight;
	var viewWidth = realWidth * scene.get_mapScale();
	var viewHeight = realHeight * scene.get_mapScale();
	scene.setSize(viewWidth, viewHeight);
	var scale = pngWidth / viewWidth;
	var m = new openfl_geom_Matrix();
	m.scale(scale, scale);
	var map = scene.map;
	map.exportPNG(true);
	var _g = 0;
	var _g1 = scene.overlays;
	while (_g < _g1.length) {
		var overlay = _g1[_g];
		++_g;
		overlay.exportPNG(true);
	}
	var m1 = map.get_transform().get_matrix().clone();
	m1.concat(m);
	bmp.draw(map, m1, null, null, null, true);
	var _g = 0;
	var _g1 = scene.overlays;
	while (_g < _g1.length) {
		var overlay = _g1[_g];
		++_g;
		if (overlay.get_visible()) {
			var m1 = overlay.get_transform().get_matrix().clone();
			m1.concat(m);
			bmp.draw(overlay, m1, null, null, null, true);
		}
	}
	scene.setSize(scWidth, scHeight);
	scene.resetOverlays();
	map.exportPNG(false);
	com_watabou_system_Exporter.savePNG(bmp, model.name);
};
com_watabou_mfcg_export_Export.asSVG = function () {
	var city = com_watabou_mfcg_model_City.instance;
	var svg = com_watabou_mfcg_export_SvgExporter.export(city, com_watabou_mfcg_scenes_TownScene.instance);
	var name = city.name;
	com_watabou_system_Exporter.saveText("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>" + haxe_xml_Printer.print(svg.root), "" + name + ".svg", "image/svg+xml");
};
com_watabou_mfcg_export_Export.asJSON = function () {
	var city = com_watabou_mfcg_model_City.instance;
	var json = com_watabou_mfcg_export_JsonExporter.export(city);
	var name = city.name;
	com_watabou_system_Exporter.saveText(json.stringify(), "" + name + ".json", "application/json");
};
var com_watabou_mfcg_export_JsonExporter = function () { };
$hxClasses["com.watabou.mfcg.export.JsonExporter"] = com_watabou_mfcg_export_JsonExporter;
com_watabou_mfcg_export_JsonExporter.__name__ = "com.watabou.mfcg.export.JsonExporter";
com_watabou_mfcg_export_JsonExporter.export = function (model) {
	com_watabou_formats_GeoJSON.SCALE = 4;
	var buildings = [];
	var monuments = [];
	var squares = [];
	var greens = [];
	var fields = [];
	var planks = new haxe_ds_ObjectMap();
	var _g = 0;
	var _g1 = model.patches;
	while (_g < _g1.length) {
		var p = _g1[_g];
		++_g;
		var ward = p.ward;
		switch (js_Boot.getClass(ward)) {
			case com_watabou_mfcg_model_wards_Alleys:
				if (ward.group.core == p) {
					var blocks = ward.group.blocks;
					switch (com_watabou_system_State.get("display_mode", "Lots")) {
						case "Block":
							var _g2 = [];
							var _g3 = 0;
							while (_g3 < blocks.length) {
								var b = blocks[_g3];
								++_g3;
								_g2.push(b.shape);
							}
							var b1 = _g2;
							var _g4 = 0;
							while (_g4 < b1.length) {
								var e = b1[_g4];
								++_g4;
								buildings.push(e);
							}
							break;
						case "Complex":
							var _g5 = [];
							var _g6 = 0;
							while (_g6 < blocks.length) {
								var b2 = blocks[_g6];
								++_g6;
								_g5.push(b2.buildings);
							}
							var b3 = com_watabou_utils_ArrayExtender.collect(_g5);
							var _g7 = 0;
							while (_g7 < b3.length) {
								var e1 = b3[_g7];
								++_g7;
								buildings.push(e1);
							}
							break;
						case "Lots":
							var _g8 = [];
							var _g9 = 0;
							while (_g9 < blocks.length) {
								var b4 = blocks[_g9];
								++_g9;
								_g8.push(b4.lots);
							}
							var b5 = com_watabou_utils_ArrayExtender.collect(_g8);
							var _g10 = 0;
							while (_g10 < b5.length) {
								var e2 = b5[_g10];
								++_g10;
								buildings.push(e2);
							}
							break;
						case "Simple":
							var _g11 = [];
							var _g12 = 0;
							while (_g12 < blocks.length) {
								var b6 = blocks[_g12];
								++_g12;
								_g11.push(b6.rects);
							}
							var b7 = com_watabou_utils_ArrayExtender.collect(_g11);
							var _g13 = 0;
							while (_g13 < b7.length) {
								var e3 = b7[_g13];
								++_g13;
								buildings.push(e3);
							}
							break;
					}
				}
				break;
			case com_watabou_mfcg_model_wards_Castle:
				buildings.push((js_Boot.__cast(ward, com_watabou_mfcg_model_wards_Castle)).building);
				break;
			case com_watabou_mfcg_model_wards_Cathedral:
				var b8 = (js_Boot.__cast(ward, com_watabou_mfcg_model_wards_Cathedral)).building;
				var _g14 = 0;
				while (_g14 < b8.length) {
					var e4 = b8[_g14];
					++_g14;
					buildings.push(e4);
				}
				break;
			case com_watabou_mfcg_model_wards_Farm:
				var farm = js_Boot.__cast(ward, com_watabou_mfcg_model_wards_Farm);
				var b9 = farm.buildings;
				var _g15 = 0;
				while (_g15 < b9.length) {
					var e5 = b9[_g15];
					++_g15;
					buildings.push(e5);
				}
				var b10 = farm.subPlots;
				var _g16 = 0;
				while (_g16 < b10.length) {
					var e6 = b10[_g16];
					++_g16;
					fields.push(e6);
				}
				break;
			case com_watabou_mfcg_model_wards_Harbour:
				var harbour = js_Boot.__cast(ward, com_watabou_mfcg_model_wards_Harbour);
				var _g17 = 0;
				var _g18 = harbour.piers;
				while (_g17 < _g18.length) {
					var pier = _g18[_g17];
					++_g17;
					planks.set(pier, 1.2);
				}
				break;
			case com_watabou_mfcg_model_wards_Market:
				var market = js_Boot.__cast(ward, com_watabou_mfcg_model_wards_Market);
				squares.push(market.getAvailable());
				if (market.monument != null) {
					monuments.push(market.monument);
				}
				break;
			case com_watabou_mfcg_model_wards_Park:
				greens.push((js_Boot.__cast(ward, com_watabou_mfcg_model_wards_Park)).green);
				break;
		}
	}
	var _g = [];
	var _g1 = 0;
	var _g2 = model.districts;
	while (_g1 < _g2.length) {
		var d = _g2[_g1];
		++_g1;
		var poly = com_watabou_formats_GeoJSON.polygon(null, com_watabou_geom_EdgeChain.toPoly(d.border));
		var _g3 = new haxe_ds_StringMap();
		_g3.h["name"] = d.name;
		poly.props = _g3;
		_g.push(poly);
	}
	var districts = com_watabou_formats_GeoJSON.geometryCollection("districts", _g);
	var _g = new haxe_ds_StringMap();
	_g.h["id"] = "values";
	_g.h["roadWidth"] = 2.0 * com_watabou_formats_GeoJSON.SCALE;
	_g.h["towerRadius"] = com_watabou_mfcg_model_CurtainWall.TOWER_RADIUS * com_watabou_formats_GeoJSON.SCALE;
	_g.h["wallThickness"] = com_watabou_mfcg_model_CurtainWall.THICKNESS * com_watabou_formats_GeoJSON.SCALE;
	_g.h["generator"] = "mfcg";
	_g.h["version"] = lime_app_Application.current.meta.h["version"];
	var values = _g;
	if (model.canals.length > 0) {
		var v = model.canals[0].width * com_watabou_formats_GeoJSON.SCALE;
		values.h["riverWidth"] = v;
	}
	var _g = 0;
	var _g1 = model.canals;
	while (_g < _g1.length) {
		var c = _g1[_g];
		++_g;
		var bridge = c.bridges.keys();
		while (bridge.hasNext()) {
			var bridge1 = bridge.next();
			if (c.bridges.h[bridge1.__id__] == null) {
				var length = c.width + 1.2;
				var course = c.course;
				var pos = com_watabou_geom_EdgeChain.indexByOrigin(course, bridge1);
				var _this = course[pos];
				var flow = _this.next.origin.point.subtract(_this.origin.point);
				if (pos > 0) {
					var _this1 = course[pos - 1];
					var q = _this1.next.origin.point.subtract(_this1.origin.point);
					flow.x += q.x;
					flow.y += q.y;
				}
				var p = new openfl_geom_Point(-flow.y, flow.x);
				var length1 = length / 2;
				if (length1 == null) {
					length1 = 1;
				}
				p = p.clone();
				p.normalize(length1);
				var half = p;
				var k = [bridge1.point.subtract(half), bridge1.point.add(half)];
				planks.set(k, 1.2);
			}
		}
	}
	var trees;
	if (com_watabou_system_State.get("show_trees")) {
		var _g = [];
		var _g1 = 0;
		var _g2 = model.getTrees();
		while (_g1 < _g2.length) {
			var t = _g2[_g1];
			++_g1;
			_g.push(t.c);
		}
		trees = _g;
	} else {
		trees = [];
	}
	var map = com_watabou_formats_GeoJSON.feature(null, values);
	var map1 = com_watabou_formats_GeoJSON.polygon("earth", com_watabou_geom_EdgeChain.toPoly(model.earthEdgeE));
	var map2 = com_watabou_mfcg_export_JsonExporter.multiThick("roads", model.arteries, null, function (r) {
		return com_watabou_geom_EdgeChain.toPolyline(r);
	}, function (r) {
		return 2.0;
	});
	var map3 = com_watabou_mfcg_export_JsonExporter.multiThick("walls", model.walls, true, function (w) {
		return w.shape;
	}, function (w) {
		return com_watabou_mfcg_model_CurtainWall.THICKNESS;
	});
	var map4 = com_watabou_mfcg_export_JsonExporter.multiThick("rivers", model.canals, null, function (c) {
		return com_watabou_geom_EdgeChain.toPolyline(c.course);
	}, function (c) {
		return c.width;
	});
	var _g = [];
	var p = planks.keys();
	while (p.hasNext()) {
		var p1 = p.next();
		_g.push(p1);
	}
	var map5 = com_watabou_formats_GeoJSON.featureCollection([map, map1, map2, map3, map4, com_watabou_mfcg_export_JsonExporter.multiThick("planks", _g, null, function (p) {
		return p;
	}, function (p) {
		return planks.h[p.__id__];
	}), com_watabou_formats_GeoJSON.multiPolygon("buildings", buildings), com_watabou_formats_GeoJSON.multiPolygon("prisms", monuments), com_watabou_formats_GeoJSON.multiPolygon("squares", squares), com_watabou_formats_GeoJSON.multiPolygon("greens", greens), com_watabou_formats_GeoJSON.multiPolygon("fields", fields), com_watabou_formats_GeoJSON.multiPoint("trees", trees), districts]);
	if (model.waterEdgeE.length > 0) {
		map5.items.push(com_watabou_formats_GeoJSON.multiPolygon("water", [com_watabou_geom_EdgeChain.toPoly(model.waterEdgeE)]));
	}
	return map5;
};
com_watabou_mfcg_export_JsonExporter.multiThick = function (id, objects, closed, getPoly, getThickness) {
	if (closed == null) {
		closed = false;
	}
	var _g = [];
	var _g1 = 0;
	while (_g1 < objects.length) {
		var obj = objects[_g1];
		++_g1;
		var poly = getPoly(obj);
		var string = closed ? com_watabou_formats_GeoJSON.polygon(null, poly) : com_watabou_formats_GeoJSON.lineString(null, poly);
		var _g2 = new haxe_ds_StringMap();
		var value = getThickness(obj) * com_watabou_formats_GeoJSON.SCALE;
		_g2.h["width"] = value;
		string.props = _g2;
		_g.push(string);
	}
	return com_watabou_formats_GeoJSON.geometryCollection(id, _g);
};
var com_watabou_mfcg_export_SvgExporter = function () { };
$hxClasses["com.watabou.mfcg.export.SvgExporter"] = com_watabou_mfcg_export_SvgExporter;
com_watabou_mfcg_export_SvgExporter.__name__ = "com.watabou.mfcg.export.SvgExporter";
com_watabou_mfcg_export_SvgExporter.export = function (model, scene) {
	com_watabou_formats_Sprite2SVG.substituteFont = com_watabou_mfcg_export_SvgExporter.fixFontNames;
	com_watabou_formats_Sprite2SVG.handleObject = com_watabou_mfcg_export_SvgExporter.detectEmblem;
	var realWidth = model.maxx - model.minx + 40.0;
	var realHeight = model.maxy - model.miny + 40.0;
	var svgWidth = realWidth * scene.get_mapScale();
	var svgHeight = realHeight * scene.get_mapScale();
	var oldWidth = scene.rWidth;
	var oldHeight = scene.rHeight;
	scene.setSize(svgWidth, svgHeight);
	var svg = com_watabou_formats_Sprite2SVG.create(svgWidth, svgHeight, model.waterEdge.length > 0 ? com_watabou_mfcg_mapping_Style.colorWater : com_watabou_mfcg_mapping_Style.colorPaper);
	var image = com_watabou_formats_Sprite2SVG.drawSprite(scene);
	com_watabou_formats_SVG.clearTransform(image);
	var child = com_watabou_formats_Sprite2SVG.getImports();
	svg.root.addChild(child);
	var child = com_watabou_formats_Sprite2SVG.getGradients();
	svg.root.addChild(child);
	svg.root.addChild(image);
	scene.setSize(oldWidth, oldHeight);
	return svg;
};
com_watabou_mfcg_export_SvgExporter.fixFontNames = function (fontName) {
	if (!com_watabou_mfcg_export_SvgExporter.embeddedScanned) {
		com_watabou_mfcg_export_SvgExporter.embeddedScanned = true;
		var _g = new haxe_ds_StringMap();
		var h = com_watabou_mfcg_export_SvgExporter.embedded.h;
		var _g1_h = h;
		var _g1_keys = Object.keys(h);
		var _g1_length = _g1_keys.length;
		var _g1_current = 0;
		while (_g1_current < _g1_length) {
			var key = _g1_keys[_g1_current++];
			var _g2_key = key;
			var _g2_value = _g1_h[key];
			var id = _g2_key;
			var font = _g2_value;
			var name = openfl_utils_Assets.getFont(id).name;
			_g.h[name] = { name: font.name, url: font.url, generic: font.generic };
		}
		com_watabou_mfcg_export_SvgExporter.embedded = _g;
	}
	var font = com_watabou_mfcg_export_SvgExporter.embedded.h[fontName];
	if (font != null) {
		com_watabou_formats_Sprite2SVG.addImport(font.url);
		return font.name + ", " + font.generic;
	} else {
		return com_watabou_formats_Sprite2SVG.substituteGenerics(fontName);
	}
};
com_watabou_mfcg_export_SvgExporter.detectEmblem = function (obj) {
	if (((obj) instanceof com_watabou_mfcg_scenes_overlays_Emblem) && com_watabou_mfcg_scenes_overlays_Emblem.svg != null) {
		return com_watabou_mfcg_scenes_overlays_Emblem.svg;
	} else {
		return null;
	}
};
var com_watabou_utils_StringUtils = function () { };
$hxClasses["com.watabou.utils.StringUtils"] = com_watabou_utils_StringUtils;
com_watabou_utils_StringUtils.__name__ = "com.watabou.utils.StringUtils";
com_watabou_utils_StringUtils.capitalize = function (s) {
	return HxOverrides.substr(s, 0, 1).toUpperCase() + HxOverrides.substr(s, 1, null);
};
com_watabou_utils_StringUtils.capitalizeAll = function (s) {
	var _g = [];
	var _g1 = 0;
	var _g2 = s.split(" ");
	while (_g1 < _g2.length) {
		var word = _g2[_g1];
		++_g1;
		_g.push(com_watabou_utils_StringUtils.capitalize(word));
	}
	return _g.join(" ");
};
com_watabou_utils_StringUtils.enumerate = function (a) {
	switch (a.length) {
		case 0:
			return "";
		case 1:
			return Std.string(a[0]);
		default:
			return a.slice(0, a.length - 1).join(", ") + " and " + Std.string(a[a.length - 1]);
	}
};
com_watabou_utils_StringUtils.repeat = function (s, n) {
	var _g = [];
	var _g1 = 0;
	var _g2 = n;
	while (_g1 < _g2) {
		var i = _g1++;
		_g.push(s);
	}
	return _g.join("");
};
com_watabou_utils_StringUtils.int2roman = function (i) {
	var _g = 0;
	var _g1 = com_watabou_utils_StringUtils.romanSymbols.length;
	while (_g < _g1) {
		var j = _g++;
		if (i >= com_watabou_utils_StringUtils.romanValues[j]) {
			return com_watabou_utils_StringUtils.romanSymbols[j] + com_watabou_utils_StringUtils.int2roman(i - com_watabou_utils_StringUtils.romanValues[j]);
		}
	}
	return "";
};
com_watabou_utils_StringUtils.int2words = function (i) {
	if (i == 0) {
		return "";
	}
	var _g = 0;
	var _g1 = com_watabou_utils_StringUtils.decimalWords.length;
	while (_g < _g1) {
		var j = _g++;
		if (i == com_watabou_utils_StringUtils.decimalValues[j]) {
			return com_watabou_utils_StringUtils.decimalWords[j];
		} else if (i > com_watabou_utils_StringUtils.decimalValues[j]) {
			var n = i / com_watabou_utils_StringUtils.decimalValues[j] | 0;
			var result = com_watabou_utils_StringUtils.int2words(n) + " " + com_watabou_utils_StringUtils.decimalWords[j];
			var rest = i % com_watabou_utils_StringUtils.decimalValues[j];
			if (rest == 0) {
				return result;
			} else {
				return result + " " + com_watabou_utils_StringUtils.int2words(rest);
			}
		}
	}
	return "";
};
var com_watabou_mfcg_linguistics_DistrictNames = function (model, districts) {
	this.model = model;
	this.districts = districts;
	this.nouns = com_watabou_mfcg_linguistics_DistrictNames.NOUNS.concat(com_watabou_mfcg_linguistics_DistrictNames.PLACES);
	this.adjs = com_watabou_mfcg_linguistics_DistrictNames.ADJS.slice();
	var _g = 0;
	var _g1 = com_watabou_utils_ArrayExtender.subset(com_watabou_mfcg_linguistics_DistrictNames.OCCUPATION, 5);
	while (_g < _g1.length) {
		var o = _g1[_g];
		++_g;
		this.adjs.push((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < 0.5 ? com_watabou_utils_GrammarExtender.plural(o) : o + "'s");
	}
	this.numbers = com_watabou_utils_ArrayExtender.subset(com_watabou_mfcg_linguistics_DistrictNames.NUMBERS, 2);
	var _g = [];
	var h = com_watabou_mfcg_linguistics_DistrictNames.DIRECTIONS.h;
	var d_h = h;
	var d_keys = Object.keys(h);
	var d_length = d_keys.length;
	var d_current = 0;
	while (d_current < d_length) {
		var d = d_keys[d_current++];
		_g.push(d);
	}
	this.dirs = _g;
};
$hxClasses["com.watabou.mfcg.linguistics.DistrictNames"] = com_watabou_mfcg_linguistics_DistrictNames;
com_watabou_mfcg_linguistics_DistrictNames.__name__ = "com.watabou.mfcg.linguistics.DistrictNames";
com_watabou_mfcg_linguistics_DistrictNames.getNoun = function (district) {
	var size = district.faces.length + Math.floor((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 * 3);
	if (size <= 2) {
		return "quarter";
	} else if (size < 6) {
		return "ward";
	} else if (size < 12) {
		return "district";
	} else {
		return "town";
	}
};
com_watabou_mfcg_linguistics_DistrictNames.merge = function (words) {
	var nSyll = 0;
	var _g = 0;
	while (_g < words.length) {
		var word = words[_g];
		++_g;
		if (word.indexOf("'") != -1) {
			return words.join(" ");
		}
		nSyll += com_watabou_mfcg_linguistics_Syllables.splitWord(word).length;
	}
	switch (nSyll) {
		case 0: case 1:
			return words.join("");
		case 2:
			var chance = 0.9;
			if (chance == null) {
				chance = 0.5;
			}
			if ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance) {
				return com_watabou_mfcg_linguistics_DistrictNames.fix(words);
			} else {
				return words.join(" ");
			}
			break;
		case 3:
			var chance = 0.2;
			if (chance == null) {
				chance = 0.5;
			}
			if ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance) {
				return com_watabou_mfcg_linguistics_DistrictNames.fix(words);
			} else {
				return words.join(" ");
			}
			break;
		default:
			return words.join(" ");
	}
};
com_watabou_mfcg_linguistics_DistrictNames.fix = function (words) {
	var rules = [[new EReg("hh", "i"), "h"], [new EReg("gg", "i"), "g"], [new EReg("ww", "i"), "w"], [new EReg("kc", "i"), "k"], [new EReg("yy", "i"), "iy"], [new EReg("dst", "i"), "st"], [new EReg("stt", "i"), "st"], [new EReg("([^uoaeiy])ss([^uoaeiy])", "i"), "$1s$2"], [new EReg("([a-z])\\1\\1", "i"), "$1$1"]];
	var word = words.join("");
	var _g = 0;
	while (_g < rules.length) {
		var rule = rules[_g];
		++_g;
		var before = word;
		var pattern = rule[0];
		var replace = rule[1];
		word = word.replace(pattern.r, replace);
		if (word != before) {
			haxe_Log.trace("" + before + " => " + word, { fileName: "Source/com/watabou/mfcg/linguistics/DistrictNames.hx", lineNumber: 331, className: "com.watabou.mfcg.linguistics.DistrictNames", methodName: "fix" });
		}
	}
	return word;
};
com_watabou_mfcg_linguistics_DistrictNames.prototype = {
	generate: function () {
		if (this.districts.length == 1) {
			this.districts[0].name = this.model.name;
			return;
		}
		var _g = 0;
		var _g1 = this.districts;
		while (_g < _g1.length) {
			var district = _g1[_g];
			++_g;
			if (district.name == null) {
				while (true) {
					var name;
					var _g2 = district.type;
					switch (_g2._hx_index) {
						case 0:
							var plaza = _g2.plaza;
							name = this.typeCenter(district, plaza);
							break;
						case 1:
							var castle = _g2.castle;
							name = this.typeCastle(district, castle);
							break;
						case 2:
							name = this.typeDocks(district);
							break;
						case 3:
							var bridge = _g2.bridge;
							name = this.typeBridge(district, bridge);
							break;
						case 4:
							var gate = _g2.gate;
							name = this.typeGate(district, gate);
							break;
						case 5:
							name = this.typeBank(district);
							break;
						case 6:
							name = this.typePark(district);
							break;
						default:
							name = this.typeRegular(district);
					}
					name = com_watabou_utils_StringUtils.capitalizeAll(name);
					var exists = false;
					var _g3 = 0;
					var _g4 = this.districts;
					while (_g3 < _g4.length) {
						var d = _g4[_g3];
						++_g3;
						if (d.name == name) {
							exists = true;
							break;
						}
					}
					if (!exists) {
						district.name = name;
						break;
					}
				}
			}
		}
	}
	, typeCenter: function (district, plaza) {
		var noun = com_watabou_mfcg_linguistics_DistrictNames.getNoun(district);
		var candidates = ["old " + noun];
		if (plaza != null) {
			candidates.push("trade " + noun);
		}
		var name = this.model.name;
		if (name.indexOf(" ") == -1 && name.indexOf("-") == -1) {
			candidates.push("old " + name);
		}
		return com_watabou_utils_ArrayExtender.fallOff(candidates);
	}
	, typeCastle: function (district, castle) {
		if (district.faces.length == 1) {
			return com_watabou_utils_ArrayExtender.fallOff(["castle", "citadel", "fortress"]);
		} else {
			return com_watabou_utils_ArrayExtender.fallOff(["castle", "upper", "military"]) + " " + com_watabou_mfcg_linguistics_DistrictNames.getNoun(district);
		}
	}
	, typeDocks: function (district) {
		var _gthis = this;
		var simpleName = function () {
			var chance = 0.66666666666666663;
			if (chance == null) {
				chance = 0.5;
			}
			if ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance) {
				return "the " + _gthis.docksNoun;
			} else {
				return _gthis.docksNoun + " " + com_watabou_mfcg_linguistics_DistrictNames.getNoun(district);
			}
		};
		var complexName = function () {
			var chance = 0.2;
			if (chance == null) {
				chance = 0.5;
			}
			if ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance) {
				var adj = _gthis.getDirection(district.label.pos);
				if (adj == null) {
					adj = com_watabou_utils_ArrayExtender.pick(_gthis.adjs);
				}
				return "" + adj + " " + _gthis.docksNoun;
			} else {
				return com_watabou_mfcg_linguistics_DistrictNames.merge([com_watabou_utils_ArrayExtender.pick(_gthis.adjs), _gthis.docksNoun]);
			}
		};
		if (this.docksNoun == null) {
			this.docksNoun = com_watabou_utils_ArrayExtender.fallOff(com_watabou_mfcg_linguistics_DistrictNames.PORTS);
			var count = com_watabou_utils_ArrayExtender.count(this.districts, function (d) {
				return d.type == com_watabou_mfcg_model_DistrictType.DOCKS;
			});
			if (count == 1) {
				return simpleName();
			} else {
				return complexName();
			}
		} else {
			return complexName();
		}
	}
	, typeBridge: function (district, bridge) {
		var chance = 0.1;
		if (chance == null) {
			chance = 0.5;
		}
		if ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance) {
			return this.getProper() + "'s " + com_watabou_utils_ArrayExtender.fallOff(com_watabou_mfcg_linguistics_DistrictNames.BRIDGES);
		} else {
			return com_watabou_mfcg_linguistics_DistrictNames.merge([com_watabou_utils_ArrayExtender.pick(this.adjs), com_watabou_utils_ArrayExtender.fallOff(com_watabou_mfcg_linguistics_DistrictNames.BRIDGES)]);
		}
	}
	, typeGate: function (district, gate) {
		var chance = 0.1;
		if (chance == null) {
			chance = 0.5;
		}
		if ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance) {
			return this.getProper() + "'s " + com_watabou_utils_ArrayExtender.fallOff(com_watabou_mfcg_linguistics_DistrictNames.GATES);
		} else {
			var dir = (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < 0.5 ? this.getDirection(gate.point) : null;
			var adj = dir != null ? dir : com_watabou_utils_ArrayExtender.pick(this.adjs);
			return com_watabou_mfcg_linguistics_DistrictNames.merge([adj, com_watabou_utils_ArrayExtender.fallOff(com_watabou_mfcg_linguistics_DistrictNames.GATES)]);
		}
	}
	, typeBank: function (district) {
		var adj = (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < 0.5 ? this.getDirection(district.label.pos) : null;
		if (adj == null) {
			adj = com_watabou_utils_ArrayExtender.pick(this.adjs);
		}
		return com_watabou_mfcg_linguistics_DistrictNames.merge([adj, com_watabou_utils_ArrayExtender.fallOff(com_watabou_mfcg_linguistics_DistrictNames.BANKS)]);
	}
	, typePark: function (district) {
		var count = com_watabou_utils_ArrayExtender.count(this.districts, function (d) {
			return d.type == com_watabou_mfcg_model_DistrictType.PARK;
		});
		if (count == 1 && (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < 0.5) {
			return "the " + com_watabou_utils_ArrayExtender.fallOff(com_watabou_mfcg_linguistics_DistrictNames.PARKS);
		} else {
			var chance = 0.5;
			if (chance == null) {
				chance = 0.5;
			}
			if ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance) {
				return this.getProper() + ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < 0.5 ? "'s " : " ") + com_watabou_utils_ArrayExtender.fallOff(com_watabou_mfcg_linguistics_DistrictNames.PARKS);
			} else {
				return com_watabou_mfcg_linguistics_DistrictNames.merge([com_watabou_utils_ArrayExtender.pick(this.adjs), com_watabou_utils_ArrayExtender.fallOff(com_watabou_mfcg_linguistics_DistrictNames.PARKS)]);
			}
		}
	}
	, typeRegular: function (district) {
		var chance = 0.05;
		if (chance == null) {
			chance = 0.5;
		}
		if ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance && this.numbers.length > 0) {
			var noun = com_watabou_utils_ArrayExtender.random(com_watabou_utils_ArrayExtender.difference(this.nouns, com_watabou_mfcg_linguistics_DistrictNames.PLACES));
			HxOverrides.remove(this.nouns, noun);
			return com_watabou_utils_ArrayExtender.pick(this.numbers) + " " + com_watabou_utils_GrammarExtender.plural(noun);
		} else {
			var chance = 0.1;
			if (chance == null) {
				chance = 0.5;
			}
			if ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance) {
				var noun = com_watabou_utils_ArrayExtender.random(com_watabou_utils_ArrayExtender.difference(this.nouns, com_watabou_mfcg_linguistics_DistrictNames.PLACES));
				HxOverrides.remove(this.nouns, noun);
				var chance = 0.6;
				if (chance == null) {
					chance = 0.5;
				}
				return "the " + ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance ? noun : com_watabou_utils_GrammarExtender.plural(noun));
			} else {
				var chance = 0.2;
				if (chance == null) {
					chance = 0.5;
				}
				if ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance) {
					var adj = this.getDirection(district.label.pos);
					if (adj == null) {
						adj = com_watabou_utils_ArrayExtender.pick(this.adjs);
					}
					return adj + " " + com_watabou_mfcg_linguistics_DistrictNames.getNoun(district);
				} else {
					return com_watabou_mfcg_linguistics_DistrictNames.merge([com_watabou_utils_ArrayExtender.pick(this.adjs), com_watabou_utils_ArrayExtender.pick(this.nouns)]);
				}
			}
		}
	}
	, getProper: function () {
		return com_watabou_mfcg_linguistics_Toponymy.nonsense();
	}
	, getDirection: function (point) {
		var a = Math.PI / 180 * this.model.north;
		var cosA = Math.cos(a);
		var sinA = -Math.sin(a);
		point = new openfl_geom_Point(point.x * cosA - point.y * sinA, point.y * cosA + point.x * sinA);
		var dir = null;
		var max = 0.0;
		var _g = 0;
		var _g1 = this.dirs;
		while (_g < _g1.length) {
			var d = _g1[_g];
			++_g;
			var p2 = com_watabou_mfcg_linguistics_DistrictNames.DIRECTIONS.h[d];
			var s = point.x * p2.x + point.y * p2.y;
			if (max < s) {
				max = s;
				dir = d;
			}
		}
		HxOverrides.remove(this.dirs, dir);
		return dir;
	}
	, __class__: com_watabou_mfcg_linguistics_DistrictNames
};
var com_watabou_mfcg_linguistics_Markov = function (words) {
	this.map = new haxe_ds_StringMap();
	if (com_watabou_mfcg_linguistics_Markov.phonemes == null) {
		com_watabou_mfcg_linguistics_Markov.phonemes = com_watabou_mfcg_linguistics_Syllables.VOWELS.concat(com_watabou_mfcg_linguistics_Syllables.CONSONANTS);
		com_watabou_mfcg_linguistics_Markov.phonemes.sort(function (s1, s2) {
			return s2.length - s1.length;
		});
	}
	this.source = words;
	var _g = 0;
	while (_g < words.length) {
		var word = words[_g];
		++_g;
		if (word != "") {
			var sylls = com_watabou_mfcg_linguistics_Markov.split(word.toLowerCase());
			var prev = [];
			var _g1 = 0;
			while (_g1 < sylls.length) {
				var syll = sylls[_g1];
				++_g1;
				var prevStr = prev.join("");
				if (Object.prototype.hasOwnProperty.call(this.map.h, prevStr)) {
					this.map.h[prevStr].push(syll);
				} else {
					var v = [syll];
					this.map.h[prevStr] = v;
				}
				prev.push(syll);
				if (prev.length > 2) {
					prev.shift();
				}
			}
			var prevStr1 = prev.join("");
			if (Object.prototype.hasOwnProperty.call(this.map.h, prevStr1)) {
				this.map.h[prevStr1].push("");
			} else {
				var v1 = [""];
				this.map.h[prevStr1] = v1;
			}
		}
	}
};
$hxClasses["com.watabou.mfcg.linguistics.Markov"] = com_watabou_mfcg_linguistics_Markov;
com_watabou_mfcg_linguistics_Markov.__name__ = "com.watabou.mfcg.linguistics.Markov";
com_watabou_mfcg_linguistics_Markov.split = function (word) {
	var result = [];
	while (word != "") {
		var phonemeFound = false;
		var _g = 0;
		var _g1 = com_watabou_mfcg_linguistics_Markov.phonemes;
		while (_g < _g1.length) {
			var ph = _g1[_g];
			++_g;
			if (HxOverrides.substr(word, -ph.length, null) == ph) {
				result.unshift(ph);
				word = HxOverrides.substr(word, 0, word.length - ph.length);
				phonemeFound = true;
				break;
			}
		}
		if (!phonemeFound) {
			word = HxOverrides.substr(word, 0, word.length - 1);
		}
	}
	return result;
};
com_watabou_mfcg_linguistics_Markov.prototype = {
	generate: function () {
		var result = "";
		var hist = [];
		var next = com_watabou_utils_ArrayExtender.random(this.map.h[""]);
		while (next != "") {
			result += next;
			hist.push(next);
			if (hist.length > 2) {
				hist.shift();
			}
			var this1 = this.map;
			var key = hist.join("");
			next = com_watabou_utils_ArrayExtender.random(this1.h[key]);
		}
		return result;
	}
	, __class__: com_watabou_mfcg_linguistics_Markov
};
var com_watabou_mfcg_linguistics_Syllables = function () { };
$hxClasses["com.watabou.mfcg.linguistics.Syllables"] = com_watabou_mfcg_linguistics_Syllables;
com_watabou_mfcg_linguistics_Syllables.__name__ = "com.watabou.mfcg.linguistics.Syllables";
com_watabou_mfcg_linguistics_Syllables.split = function (str) {
	var result = [];
	var _g = 0;
	var _g1 = str.split(" ");
	while (_g < _g1.length) {
		var word = _g1[_g];
		++_g;
		if (word != "") {
			result = result.concat(com_watabou_mfcg_linguistics_Syllables.splitWord(word));
		}
	}
	return result;
};
com_watabou_mfcg_linguistics_Syllables.splitWord = function (word) {
	var result = [];
	while (word.length > 0) {
		var syll = result.length == 0 && HxOverrides.substr(word, -1, null) == "e" ? com_watabou_mfcg_linguistics_Syllables.pinch(HxOverrides.substr(word, 0, word.length - 1)) + "e" : com_watabou_mfcg_linguistics_Syllables.pinch(word);
		result.unshift(syll);
		word = HxOverrides.substr(word, 0, word.length - syll.length);
		if (com_watabou_utils_ArrayExtender.every(com_watabou_mfcg_linguistics_Syllables.VOWELS, function (v) {
			return word.indexOf(v) == -1;
		})) {
			result[0] = word + result[0];
			word = "";
		}
	}
	return result;
};
com_watabou_mfcg_linguistics_Syllables.pinch = function (word) {
	var pos = word.length - 1;
	while (pos >= 0 && com_watabou_mfcg_linguistics_Syllables.VOWELS.indexOf(word.charAt(pos)) == -1) --pos;
	if (pos < 0) {
		return word;
	}
	var _g = 0;
	var _g1 = com_watabou_mfcg_linguistics_Syllables.VOWELS;
	while (_g < _g1.length) {
		var v = _g1[_g];
		++_g;
		if (HxOverrides.substr(word, pos - (v.length - 1), v.length) == v) {
			pos -= v.length;
			break;
		}
	}
	if (pos < 0) {
		return word;
	}
	var _g = 0;
	var _g1 = com_watabou_mfcg_linguistics_Syllables.CONSONANTS;
	while (_g < _g1.length) {
		var c = _g1[_g];
		++_g;
		if (HxOverrides.substr(word, pos - (c.length - 1), c.length) == c) {
			return HxOverrides.substr(word, pos - (c.length - 1), null);
		}
	}
	return HxOverrides.substr(word, pos + 1, null);
};
var com_watabou_mfcg_linguistics_Toponymy = function () { };
$hxClasses["com.watabou.mfcg.linguistics.Toponymy"] = com_watabou_mfcg_linguistics_Toponymy;
com_watabou_mfcg_linguistics_Toponymy.__name__ = "com.watabou.mfcg.linguistics.Toponymy";
com_watabou_mfcg_linguistics_Toponymy.cityName = function (city) {
	var size = com_watabou_mfcg_linguistics_Toponymy.getSize(city);
	var name;
	var weights = [2, 1 + size * 2, 1 + size * 5, 0.2, 0.2];
	var _g = [];
	var _g1 = 0;
	var _g2 = weights.length;
	while (_g1 < _g2) {
		var i = _g1++;
		_g.push(i);
	}
	switch (com_watabou_utils_ArrayExtender.weighted(_g, weights)) {
		case 0:
			var weights = [3, 3, 2, 1.0];
			var _g = [];
			var _g1 = 0;
			var _g2 = weights.length;
			while (_g1 < _g2) {
				var i = _g1++;
				_g.push(i);
			}
			switch (com_watabou_utils_ArrayExtender.weighted(_g, weights)) {
				case 0:
					name = com_watabou_utils_ArrayExtender.random(com_watabou_mfcg_linguistics_Toponymy.ADJ) + " " + com_watabou_mfcg_linguistics_Toponymy.getFeature(city);
					break;
				case 1:
					name = com_watabou_mfcg_linguistics_Toponymy.extend(city, com_watabou_utils_ArrayExtender.random(com_watabou_mfcg_linguistics_Toponymy.ADJ) + com_watabou_mfcg_linguistics_Toponymy.getFeature(city));
					break;
				case 2:
					name = com_watabou_mfcg_linguistics_Toponymy.drift(com_watabou_mfcg_linguistics_Toponymy.nonsense()) + " " + com_watabou_mfcg_linguistics_Toponymy.getFeature(city);
					break;
				case 3:
					name = com_watabou_mfcg_linguistics_Toponymy.drift(com_watabou_mfcg_linguistics_Toponymy.nonsense()) + "'s " + com_watabou_mfcg_linguistics_Toponymy.getFeature(city);
					break;
				default:
					name = "";
			}
			break;
		case 1:
			name = com_watabou_mfcg_linguistics_Toponymy.extend(city, ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < 0.5 ? com_watabou_utils_ArrayExtender.random(com_watabou_mfcg_linguistics_Toponymy.ADJ) : com_watabou_utils_ArrayExtender.random(com_watabou_mfcg_linguistics_Toponymy.NOUN)) + com_watabou_utils_ArrayExtender.random(com_watabou_mfcg_linguistics_Toponymy.NOUN));
			break;
		case 2:
			name = com_watabou_mfcg_linguistics_Toponymy.extend(city, com_watabou_mfcg_linguistics_Toponymy.nonsense());
			break;
		case 3:
			name = com_watabou_mfcg_linguistics_Toponymy.drift(com_watabou_mfcg_linguistics_Toponymy.nonsense(1)) + "-" + com_watabou_mfcg_linguistics_Toponymy.pick(com_watabou_mfcg_linguistics_Toponymy.PREP) + "-" + com_watabou_utils_StringUtils.capitalize((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < 0.5 ? com_watabou_mfcg_linguistics_Toponymy.getFeature(city, true) : com_watabou_mfcg_linguistics_Toponymy.drift(com_watabou_mfcg_linguistics_Toponymy.nonsense(1)));
			break;
		case 4:
			name = com_watabou_mfcg_linguistics_Toponymy.drift(com_watabou_mfcg_linguistics_Toponymy.nonsense(1)) + "-" + com_watabou_utils_StringUtils.capitalize(com_watabou_mfcg_linguistics_Toponymy.drift(com_watabou_mfcg_linguistics_Toponymy.nonsense()));
			break;
		default:
			name = "";
	}
	return com_watabou_utils_StringUtils.capitalizeAll(name);
};
com_watabou_mfcg_linguistics_Toponymy.getSize = function (city) {
	return com_watabou_utils_MathUtils.gate((city.inner.length - 10) / 15, 0, 1);
};
com_watabou_mfcg_linguistics_Toponymy.getFeature = function (city, strict) {
	if (strict == null) {
		strict = false;
	}
	var features = strict ? [] : com_watabou_utils_ArrayExtender.collect([com_watabou_mfcg_linguistics_Toponymy.GENERIC, com_watabou_mfcg_linguistics_Toponymy.FIELD, com_watabou_mfcg_linguistics_Toponymy.WOOD, com_watabou_mfcg_linguistics_Toponymy.CLIFF, com_watabou_mfcg_linguistics_Toponymy.SWAMP]);
	if (city.canals.length > 0) {
		com_watabou_utils_ArrayExtender.addAll(features, com_watabou_mfcg_linguistics_Toponymy.RIVER);
		if (city.shoreE.length == 0) {
			com_watabou_utils_ArrayExtender.addAll(features, com_watabou_mfcg_linguistics_Toponymy.GORGE);
		} else {
			com_watabou_utils_ArrayExtender.addAll(features, com_watabou_mfcg_linguistics_Toponymy.MOUTH);
		}
	} else {
		com_watabou_utils_ArrayExtender.addAll(features, com_watabou_mfcg_linguistics_Toponymy.HILL);
	}
	if (city.shoreE.length > 0) {
		com_watabou_utils_ArrayExtender.addAll(features, com_watabou_mfcg_linguistics_Toponymy.WATER);
		com_watabou_utils_ArrayExtender.addAll(features, com_watabou_mfcg_linguistics_Toponymy.SHORE);
		if (city.headland) {
			com_watabou_utils_ArrayExtender.addAll(features, com_watabou_mfcg_linguistics_Toponymy.HEADLAND);
		} else {
			com_watabou_utils_ArrayExtender.addAll(features, com_watabou_mfcg_linguistics_Toponymy.BAY);
		}
	} else {
		com_watabou_utils_ArrayExtender.addAll(features, com_watabou_mfcg_linguistics_Toponymy.VALLEY);
	}
	var chance = 0.66666666666666663;
	if (chance == null) {
		chance = 0.5;
	}
	var adj = (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance ? com_watabou_utils_ArrayExtender.random(com_watabou_mfcg_linguistics_Toponymy.ADJ) : com_watabou_mfcg_linguistics_Toponymy.nonsense();
	return com_watabou_utils_ArrayExtender.random(features);
};
com_watabou_mfcg_linguistics_Toponymy.drift = function (word) {
	var sylls = com_watabou_mfcg_linguistics_Syllables.split(word);
	var n = com_watabou_utils_ArrayExtender.random([2, 2, 3, 3, 3, 3, 4]);
	while (sylls.length > n) sylls.splice(Math.floor(1 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 * (sylls.length - 1 - 1)), 1);
	return sylls.join("");
};
com_watabou_mfcg_linguistics_Toponymy.filter = function (word) {
	var rules = [[new EReg("hh", "i"), "h"], [new EReg("gg", "i"), "g"], [new EReg("ww", "i"), "w"], [new EReg("dst", "i"), "st"], [new EReg("stt", "i"), "st"], [new EReg("([a-z])\\1\\1", "i"), "$1$1"]];
	var _g = 0;
	while (_g < rules.length) {
		var rule = rules[_g];
		++_g;
		var before = word;
		var pattern = rule[0];
		var replace = rule[1];
		word = word.replace(pattern.r, replace);
		if (word != before) {
			haxe_Log.trace("" + before + " => " + word, { fileName: "Source/com/watabou/mfcg/linguistics/Toponymy.hx", lineNumber: 192, className: "com.watabou.mfcg.linguistics.Toponymy", methodName: "filter" });
		}
	}
	return word;
};
com_watabou_mfcg_linguistics_Toponymy.nonsense = function (len) {
	if (len == null) {
		len = 2;
	}
	if (com_watabou_mfcg_linguistics_Toponymy.english == null) {
		var raw = openfl_utils_Assets.getText("words");
		var words = new EReg("\r?\n", "g").split(raw);
		com_watabou_mfcg_linguistics_Toponymy.english = new com_watabou_mfcg_linguistics_Markov(words);
	}
	var word;
	while (true) {
		word = com_watabou_mfcg_linguistics_Toponymy.english.generate();
		if (!(com_watabou_mfcg_linguistics_Toponymy.english.source.indexOf(word) != -1 || com_watabou_mfcg_linguistics_Syllables.splitWord(word).length < len)) {
			break;
		}
	}
	return word;
};
com_watabou_mfcg_linguistics_Toponymy.extend = function (city, name) {
	var large = com_watabou_mfcg_linguistics_Toponymy.getSize(city);
	if ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < 0.5) {
		name = com_watabou_mfcg_linguistics_Toponymy.drift(name);
	} else if ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < 0.5) {
		var mods = ["by", "cot", "gate", "ham", "ley", "shaw", "stead", "stoke", "ton", "wich"];
		if (city.wall != null) {
			com_watabou_utils_ArrayExtender.addAll(mods, ["bury", "burgh", "chester"]);
		}
		var chance = large;
		if (chance == null) {
			chance = 0.5;
		}
		if ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance) {
			com_watabou_utils_ArrayExtender.addAll(mods, ["stow"]);
		}
		if (city.waterEdge.length > 0) {
			com_watabou_utils_ArrayExtender.addAll(mods, ["port", "pool", "wick"]);
		}
		name = com_watabou_mfcg_linguistics_Toponymy.drift(name + com_watabou_utils_ArrayExtender.random(mods));
	} else {
		var village = ["town"];
		if (city.citadel != null) {
			com_watabou_utils_ArrayExtender.addAll(village, ["fort", "castle", "keep"]);
		}
		var chance = large;
		if (chance == null) {
			chance = 0.5;
		}
		if ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance) {
			com_watabou_utils_ArrayExtender.addAll(village, ["city"]);
		} else {
			com_watabou_utils_ArrayExtender.addAll(village, ["town", "hamlet"]);
		}
		if (city.waterEdge.length > 0) {
			com_watabou_utils_ArrayExtender.addAll(village, ["port", "quay", "wharf"]);
		}
		name = com_watabou_mfcg_linguistics_Toponymy.drift(name) + " " + com_watabou_utils_ArrayExtender.random(village);
	}
	return com_watabou_mfcg_linguistics_Toponymy.filter(name);
};
com_watabou_mfcg_linguistics_Toponymy.pick = function (a) {
	var words = [];
	var weights = [];
	var _g = 0;
	while (_g < a.length) {
		var w = a[_g];
		++_g;
		var parts = w.split(":");
		if (parts.length > 1) {
			weights.push(parseFloat(parts[parts.length - 1]));
			words.push(parts.slice(0, parts.length - 1).join(":"));
		} else {
			weights.push(1);
			words.push(w);
		}
	}
	return com_watabou_utils_ArrayExtender.weighted(words, weights);
};
var com_watabou_mfcg_mapping_FormalMap = function (model) {
	openfl_display_Sprite.call(this);
	this.model = model;
	com_watabou_mfcg_model_ModelDispatcher.districtsChanged.add($bind(this, this.layoutLabels));
	com_watabou_mfcg_mapping_PatchView.planMode = com_watabou_system_State.get("display_mode", "Lots");
	com_watabou_mfcg_mapping_PatchView.noStroke = !com_watabou_system_State.get("outline_buildings", true) && !com_watabou_mfcg_Main.preview;
	com_watabou_mfcg_mapping_PatchView.raised = com_watabou_system_State.get("raised", true) && !com_watabou_mfcg_Main.preview;
	com_watabou_mfcg_mapping_PatchView.watercolours = com_watabou_system_State.get("watercolours", false);
	com_watabou_mfcg_mapping_PatchView.drawSolid = com_watabou_system_State.get("draw_solids", true);
	this.towers = com_watabou_system_State.get("towers", 1);
	if (com_watabou_mfcg_mapping_Style.colorRoad != com_watabou_mfcg_mapping_Style.colorPaper) {
		var _g = 0;
		var _g1 = model.patches;
		while (_g < _g1.length) {
			var patch = _g1[_g];
			++_g;
			if (((patch.ward) instanceof com_watabou_mfcg_model_wards_Market)) {
				var square = (js_Boot.__cast(patch.ward, com_watabou_mfcg_model_wards_Market)).space;
				square = com_watabou_geom_polygons_PolyCut.enlarge(square, 1.);
				var squareView = new openfl_display_Shape();
				var g = squareView.get_graphics();
				this.addChild(squareView);
				g.beginFill(com_watabou_mfcg_mapping_Style.colorRoad);
				com_watabou_utils_GraphicsExtender.drawPolygon(g, square);
			}
		}
	}
	var _g = 0;
	var _g1 = model.arteries;
	while (_g < _g1.length) {
		var road = _g1[_g];
		++_g;
		var roadView = new openfl_display_Shape();
		this.drawRoad(roadView.get_graphics(), road);
		this.addChild(roadView);
	}
	var _g = 0;
	var _g1 = model.canals;
	while (_g < _g1.length) {
		var canal = _g1[_g];
		++_g;
		var canalView = new openfl_display_Shape();
		this.drawCanal(canalView.get_graphics(), canal);
		this.addChild(canalView);
	}
	this.patches = [];
	var _g = 0;
	var _g1 = model.patches;
	while (_g < _g1.length) {
		var patch = _g1[_g];
		++_g;
		var view = new com_watabou_mfcg_mapping_PatchView(patch);
		this.patches.push(view);
		if (view.draw()) {
			this.addChild(view);
		}
	}
	var _g = 0;
	var _g1 = this.patches;
	while (_g < _g1.length) {
		var patch = _g1[_g];
		++_g;
		this.addChild(patch.hotArea);
	}
	if (com_watabou_system_State.get("show_trees", false) && com_watabou_mfcg_mapping_PatchView.planMode != "Block") {
		this.addChild(new com_watabou_mfcg_mapping_TreesLayer(model));
	}
	var walls = new openfl_display_Shape();
	this.addChild(walls);
	if (model.wall != null) {
		this.drawWall(walls.get_graphics(), model.wall, false);
	}
	if (model.citadel != null) {
		this.drawWall(walls.get_graphics(), (js_Boot.__cast(model.citadel.ward, com_watabou_mfcg_model_wards_Castle)).wall, true);
	}
	this.labels = new openfl_display_Sprite();
	this.addChild(this.labels);
};
$hxClasses["com.watabou.mfcg.mapping.FormalMap"] = com_watabou_mfcg_mapping_FormalMap;
com_watabou_mfcg_mapping_FormalMap.__name__ = "com.watabou.mfcg.mapping.FormalMap";
com_watabou_mfcg_mapping_FormalMap.__super__ = openfl_display_Sprite;
com_watabou_mfcg_mapping_FormalMap.prototype = $extend(openfl_display_Sprite.prototype, {
	updateBounds: function (minx, maxx, miny, maxy) {
		if (this.model.shoreE.length > 0) {
			if (this.land != null) {
				this.removeChild(this.land);
			}
			this.land = new openfl_display_Shape();
			this.drawWaterbody(this.land.get_graphics(), minx, maxx, miny, maxy);
			this.addChildAt(this.land, 0);
		}
	}
	, drawWaterbody: function (g, minx, maxx, miny, maxy) {
		var isolines = com_watabou_system_State.get("isolines", true);
		var land = this.model.getTideline();
		var maxIso = 20.0;
		var cx = (minx + maxx) / 2;
		var cy = (miny + maxy) / 2;
		var _g = [];
		var _g1 = 0;
		while (_g1 < land.length) {
			var p = land[_g1];
			++_g1;
			if (this.model.horizon.indexOf(p) != -1) {
				_g.push(new openfl_geom_Point(p.x < cx ? minx - maxIso : maxx + maxIso, p.y < cy ? miny - maxIso : maxy + maxIso));
			} else {
				_g.push(new openfl_geom_Point(com_watabou_utils_MathUtils.gate(p.x, minx - maxIso, maxx + maxIso), com_watabou_utils_MathUtils.gate(p.y, miny - maxIso, maxy + maxIso)));
			}
		}
		land = _g;
		if (isolines) {
			var stroke = com_watabou_mfcg_mapping_Style.getStrokeWidth(com_watabou_mfcg_mapping_Style.strokeThin, true);
			var o = 1.0;
			var d = 0.0;
			var _g = [];
			_g.push(d += o *= 1.5);
			_g.push(d += o *= 1.5);
			_g.push(d += o *= 1.5);
			_g.push(d += o *= 1.5);
			_g.push(d += o *= 1.5);
			var iso = _g;
			while (iso.length > 0) {
				var d = iso.pop();
				g.lineStyle(d * 2, com_watabou_mfcg_mapping_Style.colorDark);
				com_watabou_utils_GraphicsExtender.drawPolygon(g, land);
				g.lineStyle((d - stroke) * 2, com_watabou_mfcg_mapping_Style.colorWater);
				com_watabou_utils_GraphicsExtender.drawPolygon(g, land);
			}
		}
		g.endFill();
		if (com_watabou_system_State.get("outline_water", true)) {
			g.beginFill(com_watabou_mfcg_mapping_Style.colorPaper);
			g.lineStyle(com_watabou_mfcg_mapping_Style.getStrokeWidth(com_watabou_mfcg_mapping_Style.strokeNormal, true), com_watabou_mfcg_mapping_Style.colorDark);
		} else {
			g.beginFill(com_watabou_mfcg_mapping_Style.colorPaper);
		}
		com_watabou_utils_GraphicsExtender.drawPolygon(g, land);
		g.endFill();
	}
	, drawCanal: function (g, canal) {
		var w = canal.width;
		var w1 = canal.width + 1.2;
		var course = com_watabou_geom_EdgeChain.toPolyline(canal.course);
		var coast = this.model.shoreE.length > 0;
		var stroke = com_watabou_mfcg_mapping_Style.getStrokeWidth(com_watabou_mfcg_mapping_Style.strokeNormal, true);
		if (coast) {
			course[0] = com_watabou_geom_GeomUtils.lerp(course[0], course[1]);
		}
		var _g = [];
		var bridge = canal.bridges.keys();
		while (bridge.hasNext()) {
			var bridge1 = bridge.next();
			_g.push(bridge1.point);
		}
		course = com_watabou_geom_Chaikin.render(course, false, 3, _g);
		if (com_watabou_system_State.get("outline_water", true)) {
			g.lineStyle(w, com_watabou_mfcg_mapping_Style.colorDark, null, false, null, 0);
			com_watabou_utils_GraphicsExtender.drawPolyline(g, course);
			g.lineStyle(w - stroke * 2, com_watabou_mfcg_mapping_Style.colorWater);
			com_watabou_utils_GraphicsExtender.drawPolyline(g, course);
		} else {
			g.lineStyle(w, com_watabou_mfcg_mapping_Style.colorWater);
			com_watabou_utils_GraphicsExtender.drawPolyline(g, course);
		}
		if (com_watabou_system_State.get("isolines", true)) {
			var rapid = w * 0.6;
			g.lineStyle(rapid, com_watabou_mfcg_mapping_Style.colorDark, null, false, null, 0);
			com_watabou_utils_GraphicsExtender.drawPolyline(g, course);
			g.lineStyle(rapid - com_watabou_mfcg_mapping_Style.getStrokeWidth(com_watabou_mfcg_mapping_Style.strokeThin, true) * 2, com_watabou_mfcg_mapping_Style.colorWater);
			com_watabou_utils_GraphicsExtender.drawPolyline(g, course);
		}
		if (coast) {
			this.drawMouth(g, canal);
		}
		var bridge = canal.bridges.keys();
		while (bridge.hasNext()) {
			var bridge1 = bridge.next();
			var road = canal.bridges.h[bridge1.__id__];
			if (road == null) {
				var pos = com_watabou_geom_EdgeChain.indexByOrigin(canal.course, bridge1);
				var _this = canal.course[pos];
				var flow = _this.next.origin.point.subtract(_this.origin.point);
				if (pos > 0) {
					var _this1 = canal.course[pos - 1];
					var q = _this1.next.origin.point.subtract(_this1.origin.point);
					flow.x += q.x;
					flow.y += q.y;
				}
				var p = new openfl_geom_Point(-flow.y, flow.x);
				var length = w1 / 2;
				if (length == null) {
					length = 1;
				}
				p = p.clone();
				p.normalize(length);
				var half = p;
				this.drawBridge(g, bridge1.point.subtract(half), bridge1.point.add(half), 1.2);
			} else {
				var i = com_watabou_geom_EdgeChain.indexByOrigin(road, bridge1);
				var a;
				var b;
				if (i == 0) {
					var pos1 = com_watabou_geom_EdgeChain.indexByOrigin(canal.course, bridge1);
					var _this2 = canal.course[pos1];
					var flow1 = _this2.next.origin.point.subtract(_this2.origin.point);
					var _this3 = canal.course[pos1 - 1];
					var q1 = _this3.next.origin.point.subtract(_this3.origin.point);
					flow1.x += q1.x;
					flow1.y += q1.y;
					var p1 = new openfl_geom_Point(-flow1.y, flow1.x);
					var length1 = w1 / 2;
					if (length1 == null) {
						length1 = 1;
					}
					p1 = p1.clone();
					p1.normalize(length1);
					var half1 = p1;
					var half11 = bridge1.point.add(half1);
					var half2 = bridge1.point.subtract(half1);
					var bridge2 = bridge1.point;
					var _this4 = road[i];
					var p2 = _this4.next.origin.point.subtract(_this4.origin.point);
					var length2 = w1 / 2;
					if (length2 == null) {
						length2 = 1;
					}
					p2 = p2.clone();
					p2.normalize(length2);
					a = bridge2.add(p2);
					b = openfl_geom_Point.distance(a, half11) > openfl_geom_Point.distance(a, half2) ? half11 : half2;
				} else {
					var bridge3 = bridge1.point;
					var _this5 = road[i];
					var p3 = _this5.next.origin.point.subtract(_this5.origin.point);
					var length3 = w1 / 2;
					if (length3 == null) {
						length3 = 1;
					}
					p3 = p3.clone();
					p3.normalize(length3);
					a = bridge3.add(p3);
					var bridge4 = bridge1.point;
					var _this6 = road[i - 1];
					var p4 = _this6.next.origin.point.subtract(_this6.origin.point);
					var length4 = w1 / 2;
					if (length4 == null) {
						length4 = 1;
					}
					p4 = p4.clone();
					p4.normalize(length4);
					b = bridge4.subtract(p4);
				}
				this.drawBridge(g, a, b, 2.0);
			}
		}
	}
	, drawBridge: function (g, a, b, width) {
		if (com_watabou_system_State.get("outline_roads", true) || com_watabou_system_State.get("outline_water", true)) {
			var stroke = com_watabou_mfcg_mapping_Style.getStrokeWidth(com_watabou_mfcg_mapping_Style.strokeNormal, true);
			g.lineStyle(width + stroke, com_watabou_mfcg_mapping_Style.colorDark, null, false, null, 0);
			g.moveTo(a.x, a.y);
			g.lineTo(b.x, b.y);
			g.lineStyle(width - stroke, com_watabou_mfcg_mapping_Style.colorRoad);
			g.moveTo(a.x, a.y);
			g.lineTo(b.x, b.y);
		} else {
			g.lineStyle(width, com_watabou_mfcg_mapping_Style.colorRoad);
			g.moveTo(a.x, a.y);
			g.lineTo(b.x, b.y);
		}
	}
	, drawMouth: function (g, canal) {
		var stroke = com_watabou_mfcg_mapping_Style.getStrokeWidth(com_watabou_mfcg_mapping_Style.strokeNormal, true);
		var w = canal.width - stroke;
		var mouth = canal.course[0].origin.point;
		var shore = this.model.shore;
		var index = shore.indexOf(mouth);
		var _this = canal.course[0];
		var p = _this.next.origin.point.subtract(_this.origin.point);
		var flow = new openfl_geom_Point(p.x * 0.5, p.y * 0.5);
		var _this = canal.course[0];
		var widening = com_watabou_geom_GeomUtils.lerp(_this.origin.point, _this.next.origin.point, 0.5);
		var p = new openfl_geom_Point(-flow.y, flow.x);
		var length = w / 2;
		if (length == null) {
			length = 1;
		}
		p = p.clone();
		p.normalize(length);
		var p1 = widening.add(p);
		var p2 = com_watabou_geom_GeomUtils.lerp(shore[(index + shore.length - 1) % shore.length], mouth);
		var c1 = p1.subtract(flow);
		var c2 = com_watabou_geom_GeomUtils.lerp(p2, mouth);
		var p = new openfl_geom_Point(-flow.y, flow.x);
		var length = w / 2;
		if (length == null) {
			length = 1;
		}
		p = p.clone();
		p.normalize(length);
		var p3 = widening.subtract(p);
		var c3 = p3.subtract(flow);
		var p4 = com_watabou_geom_GeomUtils.lerp(shore[(index + 1) % shore.length], mouth);
		var c4 = com_watabou_geom_GeomUtils.lerp(p4, mouth);
		g.endFill();
		g.beginFill(com_watabou_mfcg_mapping_Style.colorWater);
		g.moveTo(p1.x, p1.y);
		g.cubicCurveTo(c1.x, c1.y, c2.x, c2.y, p2.x, p2.y);
		if (com_watabou_geom_polygons_PolyAccess.isConvexVertexi(shore, index)) {
			g.lineTo(mouth.x, mouth.y);
		}
		g.lineTo(p4.x, p4.y);
		g.cubicCurveTo(c4.x, c4.y, c3.x, c3.y, p3.x, p3.y);
		g.endFill();
		if (com_watabou_system_State.get("outline_water", true)) {
			g.lineStyle(stroke, com_watabou_mfcg_mapping_Style.colorDark);
			g.moveTo(p1.x, p1.y);
			g.cubicCurveTo(c1.x, c1.y, c2.x, c2.y, p2.x, p2.y);
			g.moveTo(p3.x, p3.y);
			g.cubicCurveTo(c3.x, c3.y, c4.x, c4.y, p4.x, p4.y);
			g.moveTo(0, 0);
		}
	}
	, drawRoad: function (g, road) {
		var poly = com_watabou_geom_EdgeChain.toPolyline(road);
		var exclude = [];
		var lastUrban = 0;
		var _g = 1;
		var _g1 = road.length;
		while (_g < _g1) {
			var i = _g++;
			var v = road[i];
			var urban = true;
			var rural = true;
			var _g2 = 0;
			var _g3 = v.origin.edges;
			while (_g2 < _g3.length) {
				var e = _g3[_g2];
				++_g2;
				if (e.face.data.withinCity) {
					rural = false;
				} else {
					urban = false;
				}
			}
			if (urban) {
				lastUrban = i;
			}
			if (!rural) {
				exclude.push(v.origin.point);
			}
		}
		var round = function (poly, turn, r) {
			var i = poly.indexOf(turn);
			if (i == -1) {
				return;
			}
			var pa = null;
			var pb = null;
			if (i < poly.length - 1) {
				var p1 = poly[i + 1];
				var len1 = openfl_geom_Point.distance(turn, p1);
				if (len1 <= r) {
					return;
				}
				pb = com_watabou_geom_GeomUtils.lerp(p1, turn, (len1 - r) / len1);
			}
			if (i > 0) {
				var p0 = poly[i - 1];
				var len0 = openfl_geom_Point.distance(p0, turn);
				if (len0 <= r) {
					return;
				}
				pa = com_watabou_geom_GeomUtils.lerp(p0, turn, (len0 - r) / len0);
			}
			poly.splice(i, 1);
			if (pb != null) {
				poly.splice(i, 0, pb);
			}
			if (pa != null) {
				poly.splice(i, 0, pa);
			}
		};
		var roundAll = function (poly, turns) {
			var _g = 0;
			while (_g < exclude.length) {
				var p = exclude[_g];
				++_g;
				round(poly, p, 1.);
			}
		};
		if (com_watabou_system_State.get("outline_roads", true)) {
			var outline = com_watabou_mfcg_mapping_Style.getStrokeWidth(com_watabou_mfcg_mapping_Style.strokeNormal, true) * 2;
			var smoothed = com_watabou_geom_Chaikin.render(poly.slice(lastUrban), false, 3, exclude);
			roundAll(smoothed, exclude);
			g.lineStyle(2.0 + outline, com_watabou_mfcg_mapping_Style.colorDark, null, null, null, 0);
			com_watabou_utils_GraphicsExtender.drawPolyline(g, smoothed);
		}
		var smoothed = com_watabou_geom_Chaikin.render(poly, false, 3, exclude);
		roundAll(smoothed, exclude);
		g.lineStyle(2.0, com_watabou_mfcg_mapping_Style.colorRoad);
		com_watabou_utils_GraphicsExtender.drawPolyline(g, smoothed);
	}
	, drawWall: function (g, wall, large) {
		var wallRanges = [];
		var gateSegments = [];
		var curRange = null;
		var start = -1;
		var _g = 0;
		var _g1 = wall.edges.length;
		while (_g < _g1) {
			var i = _g++;
			var v = wall.edges[i].origin;
			if (wall.gates.indexOf(v) == -1 && wall.watergates.h.__keys__[v.__id__] == null) {
				start = i;
				break;
			}
		}
		curRange = [wall.shape[start]];
		wallRanges.push(curRange);
		var shortestSegment = Infinity;
		var i0 = start;
		while (true) {
			var i1 = (i0 + 1) % wall.edges.length;
			var e0 = wall.edges[i0];
			var e1 = wall.edges[i1];
			var l = openfl_geom_Point.distance(e0.origin.point, e0.next.origin.point);
			if (l < shortestSegment) {
				shortestSegment = l;
			}
			var v1 = e1.origin;
			var p1 = v1.point;
			var gateWidth = wall.gates.indexOf(v1) != -1 ? 2.0 : wall.watergates.h.__keys__[v1.__id__] != null ? wall.watergates.h[v1.__id__].width : 0.0;
			if (gateWidth > 0) {
				var vec0 = e0.next.origin.point.subtract(e0.origin.point);
				var vec1 = e1.next.origin.point.subtract(e1.origin.point);
				var p = vec0;
				var length = gateWidth / 2;
				if (length == null) {
					length = 1;
				}
				p = p.clone();
				p.normalize(length);
				var gate1 = p1.subtract(p);
				var p2 = vec1;
				var length1 = gateWidth / 2;
				if (length1 == null) {
					length1 = 1;
				}
				p2 = p2.clone();
				p2.normalize(length1);
				var gate2 = p1.add(p2);
				gateSegments.push(new com_watabou_geom_Segment(gate1, gate2));
				var p3 = vec0;
				var length2 = com_watabou_mfcg_model_CurtainWall.TOWER_RADIUS;
				if (length2 == null) {
					length2 = 1;
				}
				p3 = p3.clone();
				p3.normalize(length2);
				curRange.push(gate1.subtract(p3));
				var p4 = vec1;
				var length3 = com_watabou_mfcg_model_CurtainWall.TOWER_RADIUS;
				if (length3 == null) {
					length3 = 1;
				}
				p4 = p4.clone();
				p4.normalize(length3);
				curRange = [gate2.add(p4)];
				wallRanges.push(curRange);
			} else if (wall.segments[i0]) {
				curRange.push(p1);
			} else {
				curRange = [p1];
				wallRanges.push(curRange);
			}
			i0 = i1;
			if (!(i0 != start)) {
				break;
			}
		}
		g.lineStyle(com_watabou_mfcg_model_CurtainWall.THICKNESS, com_watabou_mfcg_mapping_Style.colorWall);
		var _g = 0;
		while (_g < wallRanges.length) {
			var range = wallRanges[_g];
			++_g;
			com_watabou_utils_GraphicsExtender.drawPolyline(g, range);
		}
		var _g = 0;
		var _g1 = gateSegments.length;
		while (_g < _g1) {
			var i = _g++;
			this.drawGate(g, gateSegments[i]);
		}
		var _g = 0;
		var _g1 = wall.towers;
		while (_g < _g1.length) {
			var t = _g1[_g];
			++_g;
			this.drawNormalTower(g, wall, t.point, large ? com_watabou_mfcg_model_CurtainWall.LTOWER_RADIUS : com_watabou_mfcg_model_CurtainWall.TOWER_RADIUS);
		}
		if (large) {
			shortestSegment = Math.max(shortestSegment, com_watabou_mfcg_model_CurtainWall.LTOWER_RADIUS * 3);
			var _g = 0;
			var _g1 = wall.edges;
			while (_g < _g1.length) {
				var e = _g1[_g];
				++_g;
				if (openfl_geom_Point.distance(e.origin.point, e.next.origin.point) > shortestSegment * 2.5) {
					this.drawFakeTower(g, e.origin.point, e.next.origin.point, com_watabou_mfcg_model_CurtainWall.LTOWER_RADIUS);
				}
			}
		}
	}
	, drawNormalTower: function (g, wall, p, r) {
		var outward = null;
		var convex = true;
		if (openfl_display_CapsStyle.fromString(this.towers) != 1) {
			var shape = wall.shape;
			var length = shape.length;
			var i = shape.indexOf(p);
			if (wall.bothSegments(i)) {
				var prev = shape[(i + length - 1) % length];
				var next = shape[(i + 1) % length];
				var v0 = prev.subtract(p);
				v0.normalize(1);
				var v1 = next.subtract(p);
				v1.normalize(1);
				convex = v0.x * v1.y - v0.y * v1.x < 0;
				var p1 = v0.add(v1);
				var length1 = convex ? -1 : 1;
				if (length1 == null) {
					length1 = 1;
				}
				p1 = p1.clone();
				p1.normalize(length1);
				outward = p1;
			} else {
				if (wall.segments[i]) {
					var i1 = (i + length - 1) % length;
					outward = shape[i1 < shape.length - 1 ? i1 + 1 : 0].subtract(shape[i1]);
				} else {
					outward = shape[i < shape.length - 1 ? i + 1 : 0].subtract(shape[i]);
				}
				var norm = true;
				if (norm == null) {
					norm = false;
				}
				outward.setTo(outward.y, -outward.x);
				if (norm) {
					outward.normalize(1);
				}
			}
		}
		this.drawTower(g, p, outward, convex, r);
	}
	, drawFakeTower: function (g, v0, v1, r) {
		var point = com_watabou_geom_GeomUtils.lerp(v0, v1);
		var outward = v1.subtract(v0);
		var norm = true;
		if (norm == null) {
			norm = false;
		}
		outward.setTo(outward.y, -outward.x);
		if (norm) {
			outward.normalize(1);
		}
		this.drawTower(g, point, outward, false, r);
	}
	, drawTower: function (g, point, outward, convex, r) {
		g.endFill();
		g.beginFill(com_watabou_mfcg_mapping_Style.colorWall);
		if (this.towers == "Round") {
			g.drawCircle(point.x, point.y, r);
		} else if (this.towers == "Open") {
			r *= 0.7;
			var t = r / 2;
			var point1 = new openfl_geom_Point(point.x + outward.x * t, point.y + outward.y * t);
			g.drawCircle(point1.x, point1.y, r);
			var box = com_watabou_geom_polygons_PolyCreate.rect(r * 2, r * 2);
			com_watabou_geom_polygons_PolyTransform.asTranslate(box, -r, 0);
			com_watabou_geom_polygons_PolyTransform.asRotateYX(box, outward.y, outward.x);
			g.beginFill(com_watabou_mfcg_mapping_Style.colorWall);
			com_watabou_utils_GraphicsExtender.drawPolygonAt(g, box, point1.x, point1.y);
		} else {
			var poly = this.towers == "Bastions" ? com_watabou_geom_polygons_PolyCreate.regular(3, 1.5 * r) : com_watabou_geom_polygons_PolyCreate.regular(4, 1.2 * r, Math.PI / 4);
			com_watabou_geom_polygons_PolyTransform.asRotateYX(poly, outward.y, outward.x);
			var t = (r - com_watabou_mfcg_model_CurtainWall.THICKNESS / 2) * (convex ? 0.5 : 1.5);
			var point1 = new openfl_geom_Point(point.x + outward.x * t, point.y + outward.y * t);
			com_watabou_utils_GraphicsExtender.drawPolygonAt(g, poly, point1.x, point1.y);
		}
		g.endFill();
	}
	, drawGate: function (g, gate) {
		var p = gate.end.subtract(gate.start);
		var length = com_watabou_mfcg_model_CurtainWall.TOWER_RADIUS * 2;
		if (length == null) {
			length = 1;
		}
		p = p.clone();
		p.normalize(length);
		var gateOfs = p;
		g.lineStyle(com_watabou_mfcg_model_CurtainWall.TOWER_RADIUS * 2, com_watabou_mfcg_mapping_Style.colorWall, null, null, null, 0);
		var p = gate.start.subtract(gateOfs);
		g.moveTo(p.x, p.y);
		var p = gate.start;
		g.lineTo(p.x, p.y);
		var p = gate.end;
		g.moveTo(p.x, p.y);
		var p = gate.end.add(gateOfs);
		g.lineTo(p.x, p.y);
	}
	, layoutLabels: function () {
		this.labels.removeChildren();
		if (!com_watabou_mfcg_Main.preview) {
			var _g = 0;
			var _g1 = this.model.districts;
			while (_g < _g1.length) {
				var d = _g1[_g];
				++_g;
				this.labels.addChild(new com_watabou_mfcg_ui_DistrictLabel(d));
			}
		}
	}
	, showLabels: function (state) {
		this.labels.set_visible(state);
	}
	, exportPNG: function (state) {
		var _g = 0;
		var _g1 = this.labels.get_numChildren();
		while (_g < _g1) {
			var i = _g++;
			var label = this.labels.getChildAt(i);
			label.filterOn(!state);
		}
	}
	, __class__: com_watabou_mfcg_mapping_FormalMap
});
var openfl_display_Shape = function () {
	openfl_display_DisplayObject.call(this);
	this.__drawableType = 3;
};
$hxClasses["openfl.display.Shape"] = openfl_display_Shape;
openfl_display_Shape.__name__ = "openfl.display.Shape";
openfl_display_Shape.__super__ = openfl_display_DisplayObject;
openfl_display_Shape.prototype = $extend(openfl_display_DisplayObject.prototype, {
	get_graphics: function () {
		if (this.__graphics == null) {
			this.__graphics = new openfl_display_Graphics(this);
		}
		return this.__graphics;
	}
	, __class__: openfl_display_Shape
	, __properties__: $extend(openfl_display_DisplayObject.prototype.__properties__, { get_graphics: "get_graphics" })
});
var com_watabou_mfcg_mapping_PatchView = function (patch) {
	openfl_display_Shape.call(this);
	this.patch = patch;
	patch.view = this;
	this.hotArea = new openfl_display_Sprite();
	this.hotArea.get_graphics().beginFill(0, 0);
	com_watabou_utils_GraphicsExtender.drawPolygon(this.hotArea.get_graphics(), patch.shape);
	if (!com_watabou_mfcg_Main.preview) {
		this.hotArea.addEventListener("rollOver", $bind(this, this.onRollOver));
		this.hotArea.addEventListener("click", $bind(this, this.onClick));
	}
	this.g = this.get_graphics();
};
$hxClasses["com.watabou.mfcg.mapping.PatchView"] = com_watabou_mfcg_mapping_PatchView;
com_watabou_mfcg_mapping_PatchView.__name__ = "com.watabou.mfcg.mapping.PatchView";
com_watabou_mfcg_mapping_PatchView.__super__ = openfl_display_Shape;
com_watabou_mfcg_mapping_PatchView.prototype = $extend(openfl_display_Shape.prototype, {
	onRollOver: function (e) {
		com_watabou_mfcg_ui_Tooltip.instance.set(this.patch.ward.getLabel());
	}
	, onClick: function (e) {
		if ((e.commandKey || e.shiftKey) && this.patch.ward.getLabel() != null) {
			if (((this.patch.ward) instanceof com_watabou_mfcg_model_wards_Alleys)) {
				var patch = this.patch.ward.group.core;
				patch.reroll();
			} else {
				this.patch.reroll();
			}
		}
	}
	, draw: function () {
		this.g.clear();
		var type = js_Boot.getClass(this.patch.ward);
		switch (type) {
			case com_watabou_mfcg_model_wards_Alleys: case com_watabou_mfcg_model_wards_Castle: case com_watabou_mfcg_model_wards_Cathedral: case com_watabou_mfcg_model_wards_Market:
				var fillColor = com_watabou_mfcg_mapping_Style.colorRoof;
				var lineColor = com_watabou_mfcg_mapping_Style.colorDark;
				if (com_watabou_mfcg_mapping_PatchView.watercolours) {
					var c = this.patch.ward.getColor();
					fillColor = c;
					lineColor = com_watabou_geom_Color.lerp(lineColor, c, 0.3);
				}
				switch (type) {
					case com_watabou_mfcg_model_wards_Alleys:
						var group = (js_Boot.__cast(this.patch.ward, com_watabou_mfcg_model_wards_Alleys)).group;
						if (group.core == this.patch) {
							var mode = com_watabou_mfcg_Main.preview ? "Lots" : com_watabou_mfcg_mapping_PatchView.planMode;
							var _g = 0;
							var _g1 = group.blocks;
							while (_g < _g1.length) {
								var block = _g1[_g];
								++_g;
								var churchBlock = block == group.church;
								var roofs;
								switch (mode) {
									case "Block":
										roofs = [block.shape];
										break;
									case "Complex":
										if (block.buildings == null) {
											block.createBuildings();
										}
										roofs = block.buildings;
										break;
									case "Simple":
										if (block.rects == null) {
											block.createRects();
										}
										roofs = block.rects;
										break;
									default:
										roofs = block.lots;
								}
								this.drawRoofs(roofs, fillColor, lineColor, null, null, churchBlock);
							}
						}
						break;
					case com_watabou_mfcg_model_wards_Castle:
						if (com_watabou_mfcg_mapping_Style.colorRoad != com_watabou_mfcg_mapping_Style.colorPaper) {
							this.g.beginFill(com_watabou_mfcg_mapping_Style.colorRoad);
							com_watabou_utils_GraphicsExtender.drawPolygon(this.g, this.patch.shape);
						}
						var keep = (js_Boot.__cast(this.patch.ward, com_watabou_mfcg_model_wards_Castle)).building;
						this.drawRoofs([keep], fillColor, lineColor, com_watabou_mfcg_mapping_Style.strokeThick, 1, true);
						break;
					case com_watabou_mfcg_model_wards_Cathedral:
						var temple = (js_Boot.__cast(this.patch.ward, com_watabou_mfcg_model_wards_Cathedral)).building;
						this.drawRoofs(temple, fillColor, lineColor, com_watabou_mfcg_mapping_Style.strokeThick, 1, true);
						break;
					case com_watabou_mfcg_model_wards_Market:
						if (com_watabou_mfcg_mapping_PatchView.planMode != "Block") {
							var monument = (js_Boot.__cast(this.patch.ward, com_watabou_mfcg_model_wards_Market)).monument;
							this.drawRoofs([monument], fillColor, lineColor, com_watabou_mfcg_mapping_Style.strokeNormal, 0.0, true);
						}
						break;
				}
				break;
			case com_watabou_mfcg_model_wards_Farm:
				this.drawFarm(this.patch.ward);
				break;
			case com_watabou_mfcg_model_wards_Harbour:
				this.g.lineStyle(1.2, com_watabou_mfcg_mapping_Style.colorDark, null, false, null, 0);
				var _g = 0;
				var _g1 = (js_Boot.__cast(this.patch.ward, com_watabou_mfcg_model_wards_Harbour)).piers;
				while (_g < _g1.length) {
					var pier = _g1[_g];
					++_g;
					com_watabou_utils_GraphicsExtender.drawPolyline(this.g, pier);
				}
				break;
			case com_watabou_mfcg_model_wards_Park:
				this.drawGreen((js_Boot.__cast(this.patch.ward, com_watabou_mfcg_model_wards_Park)).green);
				break;
			default:
				return false;
		}
		return true;
	}
	, drawRoofs: function (polies, fill, line, stroke, height, solid) {
		if (solid == null) {
			solid = false;
		}
		if (height == null) {
			height = 0.5;
		}
		if (stroke == null) {
			stroke = 0.0;
		}
		if (solid && com_watabou_mfcg_mapping_PatchView.drawSolid) {
			this.drawSolids(polies);
			return;
		}
		if (stroke == 0.0) {
			stroke = com_watabou_mfcg_mapping_Style.strokeNormal;
		}
		if (com_watabou_mfcg_mapping_PatchView.noStroke) {
			line = fill;
		}
		if (!com_watabou_mfcg_mapping_PatchView.raised) {
			height = 0;
		}
		var g = this.get_graphics();
		var weathered = com_watabou_mfcg_mapping_PatchView.planMode != "Block" && com_watabou_system_State.get("weathered_roofs", false) ? com_watabou_mfcg_mapping_Style.weathering / 100 : 0.0;
		var ofs = 0.0;
		if (height > 0) {
			ofs = -1.2 * height;
			var side = com_watabou_mfcg_mapping_Style.colorWall;
			var drawStripe = function (stripe) {
				var _g = [];
				var _g1 = 0;
				while (_g1 < stripe.length) {
					var v = stripe[_g1];
					++_g1;
					_g.push(new openfl_geom_Point(v.x, v.y + ofs));
				}
				var top = _g;
				top.reverse();
				g.beginFill(side);
				g.lineStyle(com_watabou_mfcg_mapping_Style.getStrokeWidth(stroke), side);
				com_watabou_utils_GraphicsExtender.drawPolygon(g, stripe.concat(top));
			};
			var _g = 0;
			while (_g < polies.length) {
				var poly = polies[_g];
				++_g;
				var stripe = null;
				var _g1 = 0;
				var _g2 = poly.length;
				while (_g1 < _g2) {
					var i = _g1++;
					var v0 = poly[i];
					var v1 = poly[(i + 1) % poly.length];
					if (v1.x < v0.x) {
						if (stripe == null) {
							stripe = [v0, v1];
						} else {
							stripe.push(v1);
						}
					} else if (stripe != null) {
						drawStripe(stripe);
						stripe = null;
					}
				}
				if (stripe != null) {
					drawStripe(stripe);
				}
				g.endFill();
			}
		}
		var _g = 0;
		while (_g < polies.length) {
			var poly = polies[_g];
			++_g;
			var fill1 = weathered == 0.0 ? fill : com_watabou_geom_Color.scale(fill, Math.pow(2, (((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 3 * 2 - 1) * weathered));
			var line1 = com_watabou_mfcg_mapping_PatchView.noStroke ? fill1 : line;
			g.beginFill(fill1);
			g.lineStyle(com_watabou_mfcg_mapping_Style.getStrokeWidth(stroke), line1, null, null, null, null, ofs == 0.0 ? 1 : null);
			if (ofs != 0.0) {
				com_watabou_utils_GraphicsExtender.drawPolygonAt(g, poly, 0, ofs);
			} else {
				com_watabou_utils_GraphicsExtender.drawPolygon(g, poly);
			}
			g.endFill();
		}
	}
	, drawSolids: function (polies) {
		this.get_graphics().endFill();
		this.get_graphics().beginFill(com_watabou_mfcg_mapping_Style.colorWall);
		var _g = 0;
		while (_g < polies.length) {
			var poly = polies[_g];
			++_g;
			com_watabou_utils_GraphicsExtender.drawPolygon(this.get_graphics(), poly);
		}
		this.get_graphics().endFill();
	}
	, drawFarm: function (farm) {
		var thickness = com_watabou_mfcg_mapping_Style.getStrokeWidth(com_watabou_mfcg_mapping_Style.strokeNormal, true);
		var color = com_watabou_mfcg_mapping_Style.colorGreen;
		this.g.lineStyle(thickness, color, 1.0, null, null, 0);
		var _g = 0;
		var _g1 = farm.furrows;
		while (_g < _g1.length) {
			var f = _g1[_g];
			++_g;
			var g = this.g;
			var p0 = f.start;
			var p1 = f.end;
			g.moveTo(p0.x, p0.y);
			g.lineTo(p1.x, p1.y);
		}
		var _g = 0;
		var _g1 = farm.edges;
		while (_g < _g1.length) {
			var e = _g1[_g];
			++_g;
			var g = this.g;
			var p0 = e.start;
			var p1 = e.end;
			g.moveTo(p0.x, p0.y);
			g.lineTo(p1.x, p1.y);
		}
		if (com_watabou_mfcg_mapping_PatchView.planMode != "Block") {
			this.drawRoofs(farm.buildings, com_watabou_mfcg_mapping_Style.colorRoof, com_watabou_mfcg_mapping_Style.colorDark, com_watabou_mfcg_mapping_Style.strokeNormal, 0.3);
		}
	}
	, drawGreen: function (green) {
		this.g.beginFill(com_watabou_mfcg_mapping_Style.colorGreen);
		com_watabou_utils_GraphicsExtender.drawPolygon(this.g, green);
	}
	, __class__: com_watabou_mfcg_mapping_PatchView
});
var com_watabou_mfcg_mapping_Style = function () { };
$hxClasses["com.watabou.mfcg.mapping.Style"] = com_watabou_mfcg_mapping_Style;
com_watabou_mfcg_mapping_Style.__name__ = "com.watabou.mfcg.mapping.Style";
com_watabou_mfcg_mapping_Style.getStrokeWidth = function (stroke, unscaleLines) {
	if (unscaleLines == null) {
		unscaleLines = true;
	}
	if (com_watabou_mfcg_mapping_Style.thinLines) {
		stroke /= 3;
	}
	if (unscaleLines) {
		stroke *= com_watabou_mfcg_mapping_Style.lineInvScale;
	}
	return stroke;
};
com_watabou_mfcg_mapping_Style.normal = function (unscaled) {
	if (unscaled == null) {
		unscaled = true;
	}
	return com_watabou_mfcg_mapping_Style.getStrokeWidth(com_watabou_mfcg_mapping_Style.strokeNormal, unscaled);
};
com_watabou_mfcg_mapping_Style.thin = function (unscaled) {
	if (unscaled == null) {
		unscaled = true;
	}
	return com_watabou_mfcg_mapping_Style.getStrokeWidth(com_watabou_mfcg_mapping_Style.strokeThin, unscaled);
};
com_watabou_mfcg_mapping_Style.thick = function (unscaled) {
	if (unscaled == null) {
		unscaled = true;
	}
	return com_watabou_mfcg_mapping_Style.getStrokeWidth(com_watabou_mfcg_mapping_Style.strokeThick, unscaled);
};
com_watabou_mfcg_mapping_Style.getTint = function (base, i, n) {
	switch (com_watabou_mfcg_mapping_Style.tintMethod) {
		case "Brightness":
			return com_watabou_mfcg_mapping_Style.brightness(base, i, n);
		case "Spectrum":
			return com_watabou_mfcg_mapping_Style.spectrum(base, i, n);
		default:
			return com_watabou_mfcg_mapping_Style.overlay(base, i, n);
	}
};
com_watabou_mfcg_mapping_Style.spectrum = function (base, i, n) {
	var base1 = com_watabou_geom_Color.rgb2hsv(base);
	var range = 360 * (n - 1) / n * com_watabou_mfcg_mapping_Style.tintStrength / 100;
	return com_watabou_geom_Color.hsv(base1.x - range * (i / (n - 1) - 0.5), base1.y, base1.z);
};
com_watabou_mfcg_mapping_Style.brightness = function (base, i, n) {
	var base1 = com_watabou_geom_Color.rgb2hsv(base);
	var range = Math.min(base1.z, 1 - base1.z) * com_watabou_mfcg_mapping_Style.tintStrength / 50;
	return com_watabou_geom_Color.hsv(base1.x, base1.y, base1.z + range * (i / (n - 1) - 0.5));
};
com_watabou_mfcg_mapping_Style.overlay = function (base, i, n) {
	var hsv = com_watabou_geom_Color.rgb2hsv(base);
	return com_watabou_geom_Color.lerp(base, com_watabou_geom_Color.hsv(hsv.x + 360 * i / n, hsv.y, hsv.z), com_watabou_mfcg_mapping_Style.tintStrength / 100);
};
com_watabou_mfcg_mapping_Style.setPalette = function (pal, save) {
	com_watabou_mfcg_mapping_Style.colorPaper = pal.getColor("colorPaper");
	com_watabou_mfcg_mapping_Style.colorLight = pal.getColor("colorLight");
	com_watabou_mfcg_mapping_Style.colorDark = pal.getColor("colorDark");
	com_watabou_mfcg_mapping_Style.colorRoof = pal.getColor("colorRoof", com_watabou_mfcg_mapping_Style.colorLight);
	com_watabou_mfcg_mapping_Style.colorWater = pal.getColor("colorWater", com_watabou_mfcg_mapping_Style.colorPaper);
	com_watabou_mfcg_mapping_Style.colorGreen = pal.getColor("colorGreen", com_watabou_mfcg_mapping_Style.colorPaper);
	com_watabou_mfcg_mapping_Style.colorRoad = pal.getColor("colorRoad", com_watabou_mfcg_mapping_Style.colorPaper);
	com_watabou_mfcg_mapping_Style.colorWall = pal.getColor("colorWall", com_watabou_mfcg_mapping_Style.colorDark);
	com_watabou_mfcg_mapping_Style.colorTree = pal.getColor("colorTree", com_watabou_mfcg_mapping_Style.colorDark);
	com_watabou_mfcg_mapping_Style.tintMethod = pal.getString("tintMethod", com_watabou_mfcg_mapping_Style.tintMethods[0]);
	com_watabou_mfcg_mapping_Style.tintStrength = pal.getInt("tintStrength", 50);
	com_watabou_mfcg_mapping_Style.weathering = pal.getInt("weathering", 20);
	if (save) {
		com_watabou_system_State.set("colors", pal.data());
	}
};
com_watabou_mfcg_mapping_Style.getPalette = function () {
	var pal = new com_watabou_utils_Palette();
	pal.setColor("colorPaper", com_watabou_mfcg_mapping_Style.colorPaper);
	pal.setColor("colorLight", com_watabou_mfcg_mapping_Style.colorLight);
	pal.setColor("colorDark", com_watabou_mfcg_mapping_Style.colorDark);
	pal.setColor("colorRoof", com_watabou_mfcg_mapping_Style.colorRoof);
	pal.setColor("colorWater", com_watabou_mfcg_mapping_Style.colorWater);
	pal.setColor("colorGreen", com_watabou_mfcg_mapping_Style.colorGreen);
	pal.setColor("colorRoad", com_watabou_mfcg_mapping_Style.colorRoad);
	pal.setColor("colorWall", com_watabou_mfcg_mapping_Style.colorWall);
	pal.setColor("colorTree", com_watabou_mfcg_mapping_Style.colorTree);
	pal.setString("tintMethod", com_watabou_mfcg_mapping_Style.tintMethod);
	pal.setInt("tintStrength", com_watabou_mfcg_mapping_Style.tintStrength);
	pal.setInt("weathering", com_watabou_mfcg_mapping_Style.weathering);
	return pal;
};
com_watabou_mfcg_mapping_Style.fillForm = function (form) {
	form.addTab("Colors");
	form.addColor("colorPaper", "Paper", com_watabou_mfcg_mapping_Style.colorPaper);
	form.addColor("colorDark", "Ink", com_watabou_mfcg_mapping_Style.colorDark);
	form.addColor("colorRoof", "Roofs", com_watabou_mfcg_mapping_Style.colorRoof);
	form.addColor("colorWater", "Water", com_watabou_mfcg_mapping_Style.colorWater);
	form.addColor("colorGreen", "Greens", com_watabou_mfcg_mapping_Style.colorGreen);
	form.addColor("colorRoad", "Roads", com_watabou_mfcg_mapping_Style.colorRoad);
	form.addColor("colorWall", "Walls", com_watabou_mfcg_mapping_Style.colorWall);
	form.addColor("colorTree", "Trees", com_watabou_mfcg_mapping_Style.colorTree);
	form.addColor("colorLight", "Compass", com_watabou_mfcg_mapping_Style.colorLight);
	form.addTab("Tints");
	form.addEnum("tintMethod", "Method", com_watabou_mfcg_mapping_Style.tintMethods, com_watabou_mfcg_mapping_Style.tintMethod);
	form.addInt("tintStrength", "Strength(%)", com_watabou_mfcg_mapping_Style.tintStrength, 0, 100);
	form.addInt("weathering", "Weathering(%)", com_watabou_mfcg_mapping_Style.weathering, 0, 100);
};
com_watabou_mfcg_mapping_Style.restore = function () {
	com_watabou_mfcg_mapping_Style.thinLines = com_watabou_system_State.get("thin_lines", true);
	var palette = com_watabou_system_State.get("colors");
	if (palette != null) {
		com_watabou_mfcg_mapping_Style.setPalette(com_watabou_utils_Palette.fromData(palette), false);
	}
};
var com_watabou_mfcg_mapping_TreesLayer = function (model) {
	openfl_display_Sprite.call(this);
	var circles = [];
	var _g = 0;
	var _g1 = model.getTrees();
	while (_g < _g1.length) {
		var t = _g1[_g];
		++_g;
		var a0 = (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647;
		var a = Math.PI * 2 * (a0 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 / 4);
		var r = t.r * (1 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 4;
		var c = t.c.add(openfl_geom_Point.polar(r, a));
		circles.push(new com_watabou_geom_Circle(c, r));
		var a1 = Math.PI * 2 * (a0 + (1 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 4);
		var r1 = t.r * (1 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 4;
		var c1 = t.c.add(openfl_geom_Point.polar(r1, a1));
		circles.push(new com_watabou_geom_Circle(c1, r1));
		var a2 = Math.PI * 2 * (a0 + (2 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 4);
		var r2 = t.r * (1 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 4;
		var c2 = t.c.add(openfl_geom_Point.polar(r2, a2));
		circles.push(new com_watabou_geom_Circle(c2, r2));
		var a3 = Math.PI * 2 * (a0 + (3 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 4);
		var r3 = t.r * (1 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 4;
		var c3 = t.c.add(openfl_geom_Point.polar(r3, a3));
		circles.push(new com_watabou_geom_Circle(c3, r3));
	}
	var g = this.get_graphics();
	if (com_watabou_system_State.get("outline_trees", true)) {
		var halfStr = com_watabou_mfcg_mapping_Style.getStrokeWidth(com_watabou_mfcg_mapping_Style.strokeNormal, true);
		var _g = 0;
		while (_g < circles.length) {
			var t = circles[_g];
			++_g;
			g.beginFill(com_watabou_mfcg_mapping_Style.colorDark);
			g.drawCircle(t.c.x, t.c.y, t.r + halfStr);
		}
	}
	var _g = 0;
	while (_g < circles.length) {
		var t = circles[_g];
		++_g;
		g.beginFill(com_watabou_mfcg_mapping_Style.colorTree);
		g.drawCircle(t.c.x, t.c.y, t.r);
	}
	this.mouseEnabled = false;
};
$hxClasses["com.watabou.mfcg.mapping.TreesLayer"] = com_watabou_mfcg_mapping_TreesLayer;
com_watabou_mfcg_mapping_TreesLayer.__name__ = "com.watabou.mfcg.mapping.TreesLayer";
com_watabou_mfcg_mapping_TreesLayer.__super__ = openfl_display_Sprite;
com_watabou_mfcg_mapping_TreesLayer.prototype = $extend(openfl_display_Sprite.prototype, {
	__class__: com_watabou_mfcg_mapping_TreesLayer
});
var com_watabou_mfcg_model_Blueprint = function (size, seed) {
	this.export = null;
	this.coastDir = NaN;
	this.size = size;
	this.seed = seed;
};
$hxClasses["com.watabou.mfcg.model.Blueprint"] = com_watabou_mfcg_model_Blueprint;
com_watabou_mfcg_model_Blueprint.__name__ = "com.watabou.mfcg.model.Blueprint";
com_watabou_mfcg_model_Blueprint.create = function (size, seed) {
	var bp = new com_watabou_mfcg_model_Blueprint(size, seed);
	bp.name = null;
	bp.pop = 0;
	bp.greens = com_watabou_system_State.get("green", false);
	bp.farms = com_watabou_system_State.get("farms", true);
	bp.random = com_watabou_system_State.get("random", true);
	if (bp.random) {
		var chance = (size + 30) / 80;
		if (chance == null) {
			chance = 0.5;
		}
		bp.walls = (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance;
		var chance = size / 80;
		if (chance == null) {
			chance = 0.5;
		}
		bp.shanty = (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance;
		var chance = 0.5 + size / 100;
		if (chance == null) {
			chance = 0.5;
		}
		bp.citadel = (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance;
		var chance = bp.walls ? size / (size + 30) : 0.5;
		if (chance == null) {
			chance = 0.5;
		}
		bp.inner = (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance;
		var chance = 0.9;
		if (chance == null) {
			chance = 0.5;
		}
		bp.plaza = (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance;
		var chance = size / 18;
		if (chance == null) {
			chance = 0.5;
		}
		bp.temple = (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance;
		var chance = 0.66666666666666663;
		if (chance == null) {
			chance = 0.5;
		}
		bp.river = (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance;
		bp.coast = (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < 0.5;
	} else {
		bp.citadel = com_watabou_system_State.get("citadel", true);
		bp.inner = com_watabou_system_State.get("urban_castle", false);
		bp.plaza = com_watabou_system_State.get("plaza", true);
		bp.temple = com_watabou_system_State.get("temple", true);
		bp.walls = com_watabou_system_State.get("walls", true);
		bp.shanty = com_watabou_system_State.get("shantytown", false);
		bp.coast = com_watabou_system_State.get("coast", true);
		bp.river = com_watabou_system_State.get("river", true);
	}
	bp.hub = com_watabou_system_State.get("hub", false);
	bp.gates = com_watabou_system_State.get("gates", -1);
	return bp;
};
com_watabou_mfcg_model_Blueprint.similar = function (another) {
	var bp = com_watabou_mfcg_model_Blueprint.create(another.size, another.seed);
	bp.name = another.name;
	return bp;
};
com_watabou_mfcg_model_Blueprint.fromURL = function () {
	var size = com_watabou_system_URLState.getInt("size", 0);
	var seed = com_watabou_system_URLState.getInt("seed", com_watabou_utils_Random.seed);
	if (size == 0 || seed == 0) {
		return null;
	}
	var bp = new com_watabou_mfcg_model_Blueprint(size, seed);
	bp.name = com_watabou_system_URLState.get("name");
	bp.pop = com_watabou_system_URLState.getInt("population", 0);
	bp.greens = com_watabou_system_URLState.getFlag("greens", false);
	bp.farms = com_watabou_system_URLState.getFlag("farms", true);
	bp.citadel = com_watabou_system_URLState.getFlag("citadel", true);
	bp.inner = com_watabou_system_URLState.getFlag("urban_castle", false);
	bp.plaza = com_watabou_system_URLState.getFlag("plaza", true);
	bp.temple = com_watabou_system_URLState.getFlag("temple", true);
	bp.walls = com_watabou_system_URLState.getFlag("walls", true);
	bp.shanty = com_watabou_system_URLState.getFlag("shantytown", false);
	bp.river = com_watabou_system_URLState.getFlag("river", false);
	bp.coast = com_watabou_system_URLState.getFlag("coast", true);
	bp.hub = com_watabou_system_URLState.getFlag("hub", false);
	bp.gates = com_watabou_system_URLState.getInt("gates", -1);
	bp.coastDir = parseFloat(com_watabou_system_URLState.get("sea", "0.0"));
	bp.export = com_watabou_system_URLState.get("export");
	return bp;
};
com_watabou_mfcg_model_Blueprint.prototype = {
	updateURL: function () {
		com_watabou_system_URLState.reset();
		com_watabou_system_URLState.set("size", this.size);
		com_watabou_system_URLState.set("seed", this.seed);
		if (this.name != null) {
			com_watabou_system_URLState.set("name", this.name);
		}
		if (this.pop > 0) {
			com_watabou_system_URLState.set("population", this.pop);
		}
		com_watabou_system_URLState.setFlag("greens", this.greens);
		com_watabou_system_URLState.setFlag("farms", this.farms);
		com_watabou_system_URLState.setFlag("citadel", this.citadel);
		com_watabou_system_URLState.setFlag("urban_castle", this.inner);
		com_watabou_system_URLState.setFlag("plaza", this.plaza);
		com_watabou_system_URLState.setFlag("temple", this.temple);
		com_watabou_system_URLState.setFlag("walls", this.walls);
		com_watabou_system_URLState.setFlag("shantytown", this.shanty);
		com_watabou_system_URLState.setFlag("coast", this.coast);
		com_watabou_system_URLState.setFlag("river", this.river);
		if (this.hub) {
			com_watabou_system_URLState.setFlag("hub");
		} else {
			com_watabou_system_URLState.set("gates", this.gates);
		}
		if (this.coast) {
			com_watabou_system_URLState.set("sea", this.coastDir);
		}
	}
	, __class__: com_watabou_mfcg_model_Blueprint
};
var com_watabou_mfcg_model_Building = function () { };
$hxClasses["com.watabou.mfcg.model.Building"] = com_watabou_mfcg_model_Building;
com_watabou_mfcg_model_Building.__name__ = "com.watabou.mfcg.model.Building";
com_watabou_mfcg_model_Building.create = function (poly, minBlockSq, front, symmetrical, chaos) {
	if (chaos == null) {
		chaos = 0.0;
	}
	if (symmetrical == null) {
		symmetrical = false;
	}
	if (front == null) {
		front = false;
	}
	var minSide = Math.sqrt(minBlockSq);
	var side0 = openfl_geom_Point.distance(poly[0], poly[1]);
	var side1 = openfl_geom_Point.distance(poly[1], poly[2]);
	var side2 = openfl_geom_Point.distance(poly[2], poly[3]);
	var side3 = openfl_geom_Point.distance(poly[3], poly[0]);
	var cols = Math.ceil(Math.min(side0, side2) / minSide);
	var rows = Math.ceil(Math.min(side1, side3) / minSide);
	if (cols <= 1 || rows <= 1) {
		return null;
	}
	var matrix = symmetrical ? com_watabou_mfcg_model_Building.getPlanSym(cols, rows) : front ? com_watabou_mfcg_model_Building.getPlanFront(cols, rows) : com_watabou_mfcg_model_Building.getPlan(cols, rows);
	var weight = 0;
	var _g = 0;
	while (_g < matrix.length) {
		var m = matrix[_g];
		++_g;
		if (m) {
			++weight;
		}
	}
	if (weight >= cols * rows) {
		return null;
	}
	var blocks = com_watabou_mfcg_utils_Cutter.grid(poly, cols, rows, chaos);
	var _g = [];
	var _g1 = 0;
	var _g2 = blocks.length;
	while (_g1 < _g2) {
		var i = _g1++;
		if (matrix[i]) {
			_g.push(blocks[i]);
		}
	}
	blocks = _g;
	return com_watabou_mfcg_model_Building.circumference(blocks);
};
com_watabou_mfcg_model_Building.getPlan = function (cols, rows, fill) {
	if (fill == null) {
		fill = 0.5;
	}
	var c = cols * rows;
	var _g = [];
	var _g1 = 0;
	var _g2 = c;
	while (_g1 < _g2) {
		var i = _g1++;
		_g.push(false);
	}
	var m = _g;
	var x = Math.floor((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 * cols);
	var y = Math.floor((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 * rows);
	m[x + y * cols] = true;
	--c;
	var minx = x;
	var maxx = x;
	var miny = y;
	var maxy = y;
	while (true) {
		var x = Math.floor((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 * cols);
		var y = Math.floor((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 * rows);
		var pos = x + y * cols;
		if (!m[pos] && (x > 0 && m[pos - 1] || y > 0 && m[pos - cols] || x < cols - 1 && m[pos + 1] || y < rows - 1 && m[pos + cols])) {
			if (minx > x) {
				minx = x;
			}
			if (maxx < x) {
				maxx = x;
			}
			if (miny > y) {
				miny = y;
			}
			if (maxy < y) {
				maxy = y;
			}
			m[pos] = true;
			--c;
		}
		var tmp;
		if (!(minx > 0 || maxx < cols - 1 || miny > 0 || maxy < rows - 1)) {
			if (c > 0) {
				var chance = fill;
				if (chance == null) {
					chance = 0.5;
				}
				tmp = (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance;
			} else {
				tmp = false;
			}
		} else {
			tmp = true;
		}
		if (!tmp) {
			break;
		}
	}
	return m;
};
com_watabou_mfcg_model_Building.getPlanFront = function (cols, rows) {
	var c = cols * rows;
	var _g = [];
	var _g1 = 0;
	var _g2 = c;
	while (_g1 < _g2) {
		var i = _g1++;
		_g.push(i < cols);
	}
	var m = _g;
	c -= cols;
	var maxy = 0;
	while (true) {
		var x = Math.floor((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 * cols);
		var y = Math.floor(1 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 * (rows - 1));
		var pos = x + y * cols;
		if (!m[pos] && (x > 0 && m[pos - 1] || y > 0 && m[pos - cols] || x < cols - 1 && m[pos + 1] || y < rows - 1 && m[pos + cols])) {
			if (maxy < y) {
				maxy = y;
			}
			m[pos] = true;
			--c;
		}
		var tmp;
		if (maxy >= rows - 1) {
			if (c > 0) {
				var chance = 0.5;
				if (chance == null) {
					chance = 0.5;
				}
				tmp = (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance;
			} else {
				tmp = false;
			}
		} else {
			tmp = true;
		}
		if (!tmp) {
			break;
		}
	}
	return m;
};
com_watabou_mfcg_model_Building.getPlanSym = function (cols, rows) {
	var plan = com_watabou_mfcg_model_Building.getPlan(cols, rows, 0);
	var _g = 0;
	var _g1 = rows;
	while (_g < _g1) {
		var i = _g++;
		var _g2 = 0;
		var _g3 = cols;
		while (_g2 < _g3) {
			var j = _g2++;
			var a = i * cols + j;
			var b = (i + 1) * cols - 1 - j;
			plan[a] = plan[b] = plan[a] || plan[b];
		}
	}
	return plan;
};
com_watabou_mfcg_model_Building.circumference = function (parts) {
	if (parts.length == 0) {
		return [];
	} else if (parts.length == 1) {
		return parts[0];
	}
	var edgeA = [];
	var edgeB = [];
	var _g = 0;
	while (_g < parts.length) {
		var p = parts[_g];
		++_g;
		var _g1 = 0;
		var _g2 = p.length;
		while (_g1 < _g2) {
			var j = _g1++;
			var a = p[j];
			var b = p[(j + 1) % p.length];
			var outer = true;
			var from = 0;
			while (true) {
				var i = edgeA.indexOf(b, from);
				if (i == -1) {
					break;
				} else if (edgeB[i] == a) {
					edgeA.splice(i, 1);
					edgeB.splice(i, 1);
					outer = false;
					break;
				} else {
					from = i + 1;
				}
				if (!(from < edgeA.length)) {
					break;
				}
			}
			if (outer) {
				edgeA.push(a);
				edgeB.push(b);
			}
		}
	}
	var loop = 0;
	var _g = 0;
	var _g1 = edgeA.length;
	while (_g < _g1) {
		var i = _g++;
		if (edgeA.lastIndexOf(edgeA[i]) != i) {
			loop = i;
			break;
		}
	}
	var start = edgeA[loop];
	var vertex = edgeB[loop];
	var poly = [start];
	while (true) {
		poly.push(vertex);
		vertex = edgeB[edgeA.indexOf(vertex)];
		if (!(vertex != start)) {
			break;
		}
	}
	return poly;
};
var com_watabou_mfcg_model_Canal = function (model, course) {
	var c = [];
	var _g = 0;
	while (_g < course.length) {
		var e = course[_g];
		++_g;
		var chance = 1 - 9 / openfl_geom_Point.distance(e.origin.point, e.next.origin.point);
		if (chance == null) {
			chance = 0.5;
		}
		if ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance) {
			c.push(e);
		} else {
			c.push(e);
		}
	}
	course = c;
	var points = com_watabou_geom_EdgeChain.toPoly(course);
	com_watabou_geom_polygons_PolyCore.set(points, com_watabou_mfcg_utils_PolyUtils.smoothOpen(points, null, 2));
	this.course = course;
};
$hxClasses["com.watabou.mfcg.model.Canal"] = com_watabou_mfcg_model_Canal;
com_watabou_mfcg_model_Canal.__name__ = "com.watabou.mfcg.model.Canal";
com_watabou_mfcg_model_Canal.createRiver = function (model) {
	com_watabou_mfcg_model_Canal.buildTopology(model);
	var course = model.shoreE.length > 0 ? com_watabou_mfcg_model_Canal.deltaRiver(model) : com_watabou_mfcg_model_Canal.regularRiver(model);
	if (course != null) {
		return new com_watabou_mfcg_model_Canal(model, course);
	} else {
		throw new openfl_errors_Error("Unable to build a canal!");
	}
};
com_watabou_mfcg_model_Canal.regularRiver = function (model) {
	var candidates = com_watabou_geom_EdgeChain.vertices(model.horizonE);
	var _g = [];
	var _g1 = 0;
	var _g2 = candidates;
	while (_g1 < _g2.length) {
		var v = _g2[_g1];
		++_g1;
		if (model.patchesByVertex(v).length > 1) {
			_g.push(v);
		}
	}
	candidates = _g;
	while (candidates.length > 1) {
		var mouth = com_watabou_utils_ArrayExtender.random(candidates);
		var source = null;
		var maxDot = Infinity;
		var _g = 0;
		while (_g < candidates.length) {
			var v = candidates[_g];
			++_g;
			var p1 = mouth.point;
			var p = v.point;
			p = p.clone();
			p.normalize(1);
			var p2 = p;
			var dot = p1.x * p2.x + p1.y * p2.y;
			if (maxDot > dot) {
				maxDot = dot;
				source = v;
			}
		}
		var centerVertex = model.dcel.vertices.h[model.center.__id__];
		var cityPoint = com_watabou_utils_ArrayExtender.random(centerVertex.edges).next.origin;
		var c1 = com_watabou_mfcg_model_Canal.topology.buildPath(source, cityPoint);
		var c2 = c1 != null ? com_watabou_mfcg_model_Canal.topology.buildPath(cityPoint, mouth) : null;
		if (c1 != null && c2 != null) {
			var _g1 = 0;
			var _g2 = c2.length;
			while (_g1 < _g2) {
				var i = _g1++;
				var j = c1.indexOf(c2[i]);
				if (j != -1) {
					var course = model.dcel.vertices2chain(c2.slice(0, i).concat(c1.slice(j)));
					if (com_watabou_mfcg_model_Canal.validateCourse(course)) {
						return course;
					} else {
						break;
					}
				}
			}
		}
		haxe_Log.trace("discard", { fileName: "Source/com/watabou/mfcg/model/Canal.hx", lineNumber: 180, className: "com.watabou.mfcg.model.Canal", methodName: "regularRiver" });
		HxOverrides.remove(candidates, mouth);
		HxOverrides.remove(candidates, source);
	}
	return null;
};
com_watabou_mfcg_model_Canal.deltaRiver = function (model) {
	var dstCandidates = [];
	var _g = 1;
	var _g1 = model.shoreE.length - 1;
	while (_g < _g1) {
		var i = _g++;
		var v = model.shoreE[i].origin;
		var split = com_watabou_utils_ArrayExtender.count(model.patchesByVertex(v), function (p) {
			return !p.waterbody;
		}) > 1;
		if (split) {
			dstCandidates.push(i);
		}
	}
	dstCandidates = com_watabou_utils_ArrayExtender.sortBy(dstCandidates, function (i) {
		return model.shoreE[i].origin.point.get_length();
	});
	var srcCandidates = com_watabou_geom_EdgeChain.vertices(com_watabou_utils_ArrayExtender.difference(model.earthEdgeE, model.shoreE));
	var _g = [];
	var _g1 = 0;
	var _g2 = srcCandidates;
	while (_g1 < _g2.length) {
		var v = _g2[_g1];
		++_g1;
		if (model.patchesByVertex(v).length > 1) {
			_g.push(v);
		}
	}
	srcCandidates = _g;
	while (dstCandidates.length > 0) {
		var idx = dstCandidates.shift();
		var mouth = model.shoreE[idx].origin;
		var prev = model.shoreE[idx - 1].origin;
		var next = model.shoreE[idx + 1].origin;
		var p = next.point.subtract(prev.point);
		p = p.clone();
		p.normalize(1);
		var p1 = p;
		var normal = new openfl_geom_Point(-p1.y, p1.x);
		var source = null;
		var maxDot = -Infinity;
		var _g = 0;
		while (_g < srcCandidates.length) {
			var v = srcCandidates[_g];
			++_g;
			var p2 = v.point.subtract(mouth.point);
			p2 = p2.clone();
			p2.normalize(1);
			var dir = p2;
			var dot = normal.x * dir.x + normal.y * dir.y;
			if (maxDot < dot) {
				maxDot = dot;
				source = v;
			}
		}
		var path = com_watabou_mfcg_model_Canal.topology.buildPath(source, mouth);
		if (path != null) {
			var course = model.dcel.vertices2chain(path);
			if (com_watabou_mfcg_model_Canal.validateCourse(course)) {
				return course;
			}
		}
		haxe_Log.trace("discard", { fileName: "Source/com/watabou/mfcg/model/Canal.hx", lineNumber: 232, className: "com.watabou.mfcg.model.Canal", methodName: "deltaRiver" });
	}
	return null;
};
com_watabou_mfcg_model_Canal.validateCourse = function (course) {
	if (course == null) {
		return false;
	}
	if (course.length < com_watabou_mfcg_model_Canal.model.earthEdge.length / 5) {
		return false;
	}
	var _g = 1;
	var _g1 = course.length - 1;
	while (_g < _g1) {
		var i = _g++;
		if (com_watabou_geom_EdgeChain.edgeByOrigin(com_watabou_mfcg_model_Canal.model.shoreE, course[i].origin) != null) {
			return false;
		}
	}
	if (com_watabou_mfcg_model_Canal.model.wall != null) {
		var wall = com_watabou_mfcg_model_Canal.model.wall.edges;
		var _g = 0;
		var _g1 = wall.length;
		while (_g < _g1) {
			var j = _g++;
			var wallj = wall[j];
			var i = com_watabou_geom_EdgeChain.indexByOrigin(course, wallj.origin);
			if (i > 0 && i < course.length - 1) {
				if (!com_watabou_mfcg_model_Canal.intersect(wall, wallj, course, course[i])) {
					return false;
				}
			}
		}
	}
	var _g = 0;
	var _g1 = com_watabou_mfcg_model_Canal.model.arteries;
	while (_g < _g1.length) {
		var road = _g1[_g];
		++_g;
		var _g2 = 1;
		var _g3 = road.length - 1;
		while (_g2 < _g3) {
			var j = _g2++;
			var roadj = road[j];
			var i = com_watabou_geom_EdgeChain.indexByOrigin(course, roadj.origin);
			if (i > 0 && i < course.length - 1) {
				if (!com_watabou_mfcg_model_Canal.intersect(road, roadj, course, course[i])) {
					return false;
				}
			}
		}
	}
	return true;
};
com_watabou_mfcg_model_Canal.intersect = function (a, a2, b, b2) {
	var a1 = com_watabou_geom_EdgeChain.prev(a, a2);
	var b1 = com_watabou_geom_EdgeChain.prev(b, b2);
	var e = a1;
	while (true) {
		e = e.next.twin;
		if (!(e != b1 && e != a2.twin && e != b2.twin)) {
			break;
		}
	}
	if (e == a2.twin) {
		return false;
	}
	while (true) {
		e = e.next.twin;
		if (!(e != b1 && e != a2.twin && e != b2.twin)) {
			break;
		}
	}
	if (e == a2.twin) {
		return true;
	} else {
		return false;
	}
};
com_watabou_mfcg_model_Canal.buildTopology = function (model) {
	if (model.patches != com_watabou_mfcg_model_Canal.patches) {
		com_watabou_mfcg_model_Canal.model = model;
		com_watabou_mfcg_model_Canal.patches = model.patches;
		var _g = [];
		var _g1 = 0;
		var _g2 = model.patches;
		while (_g1 < _g2.length) {
			var p = _g2[_g1];
			++_g1;
			if (!p.waterbody) {
				_g.push(p);
			}
		}
		com_watabou_mfcg_model_Canal.topology = new com_watabou_mfcg_model_Topology(_g);
		if (model.wall != null) {
			com_watabou_mfcg_model_Canal.topology.excludePolygon(model.wall.edges);
		}
		if (model.citadel != null) {
			com_watabou_mfcg_model_Canal.topology.excludePoints(com_watabou_geom_EdgeChain.vertices((js_Boot.__cast(model.citadel.ward, com_watabou_mfcg_model_wards_Castle)).wall.edges));
		}
		com_watabou_mfcg_model_Canal.topology.excludePoints(model.gates);
		var _g = 0;
		var _g1 = model.arteries;
		while (_g < _g1.length) {
			var a = _g1[_g];
			++_g;
			com_watabou_mfcg_model_Canal.topology.excludePolygon(a);
		}
	}
};
com_watabou_mfcg_model_Canal.prototype = {
	updateState: function () {
		this.gates = new haxe_ds_ObjectMap();
		if (com_watabou_mfcg_model_Canal.model.wall != null) {
			var _g = 0;
			var _g1 = com_watabou_mfcg_model_Canal.model.wall.edges;
			while (_g < _g1.length) {
				var e = _g1[_g];
				++_g;
				var v = e.origin;
				var i = com_watabou_geom_EdgeChain.indexByOrigin(this.course, v);
				if (i > 0 && i < this.course.length - 1) {
					com_watabou_mfcg_model_Canal.model.wall.addWatergate(v, this);
					var v1 = com_watabou_mfcg_model_Canal.model.wall;
					this.gates.set(v, v1);
				}
			}
		}
		this.bridges = new haxe_ds_ObjectMap();
		var _g = 0;
		var _g1 = com_watabou_mfcg_model_Canal.model.arteries;
		while (_g < _g1.length) {
			var road = _g1[_g];
			++_g;
			var _g2 = 0;
			var _g3 = road.length;
			while (_g2 < _g3) {
				var j = _g2++;
				var bridge = road[j];
				var i = com_watabou_geom_EdgeChain.indexByOrigin(this.course, bridge.origin);
				if (i > 0 && i < this.course.length - 1) {
					if (j == 0) {
						this.bridges.set(bridge.origin, road);
					} else if (com_watabou_mfcg_model_Canal.intersect(road, bridge, this.course, this.course[i])) {
						this.bridges.set(bridge.origin, road);
					}
				}
			}
		}
		var townPatches = com_watabou_mfcg_model_Canal.model.inner;
		var bends = [];
		var _g = 2;
		var _g1 = this.course.length - 1;
		while (_g < _g1) {
			var i = _g++;
			var edge = this.course[i];
			if (townPatches.indexOf(edge.face.data) != -1 || townPatches.indexOf(edge.twin.face.data) != -1) {
				com_watabou_utils_ArrayExtender.add(bends, edge.origin);
			}
		}
		var _g = [];
		var g = this.gates.keys();
		while (g.hasNext()) {
			var g1 = g.next();
			_g.push(g1);
		}
		com_watabou_utils_ArrayExtender.removeAll(bends, _g);
		var townBends = bends.length;
		this.rural = townBends == 0;
		this.width = (3 + com_watabou_mfcg_model_Canal.model.inner.length / 5) * (0.8 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 * 0.4) * (this.rural ? 1.5 : 1.0);
		if (!this.rural) {
			var townBridges = 0;
			var bridge = this.bridges.keys();
			while (bridge.hasNext()) {
				var bridge1 = bridge.next();
				if (bends.indexOf(bridge1) != -1) {
					++townBridges;
				}
			}
			var _g = [];
			var br = this.bridges.keys();
			while (br.hasNext()) {
				var br1 = br.next();
				_g.push(br1);
			}
			com_watabou_utils_ArrayExtender.removeAll(bends, _g);
			while (true) {
				var chance = 1 - 2 * townBridges / townBends;
				if (chance == null) {
					chance = 0.5;
				}
				if (!((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance)) {
					break;
				}
				var _g = [];
				var _g1 = 0;
				while (_g1 < bends.length) {
					var b = bends[_g1];
					++_g1;
					_g.push(1 / b.edges.length);
				}
				var bridge = com_watabou_utils_ArrayExtender.weighted(bends, _g);
				var v = null;
				this.bridges.set(bridge, v);
				HxOverrides.remove(bends, bridge);
				++townBridges;
			}
		}
		com_watabou_geom_EdgeChain.assignData(this.course, com_watabou_mfcg_model_PatchEdge.CANAL(this));
	}
	, get_mouth: function () {
		return this.course[0].origin;
	}
	, __class__: com_watabou_mfcg_model_Canal
	, __properties__: { get_mouth: "get_mouth" }
};
var com_watabou_mfcg_model_City = function (bp) {
	this.bp = bp;
	var nPatches = bp.size;
	var seed = bp.seed;
	if (nPatches == 0) {
		return;
	}
	if (seed > 0) {
		com_watabou_utils_Random.reset(seed);
	}
	if (nPatches == -1) {
		nPatches = com_watabou_mfcg_model_City.nextSize;
	}
	this.nPatches = nPatches;
	haxe_Log.trace(">> seed:" + com_watabou_utils_Random.seed + " size:" + nPatches, { fileName: "Source/com/watabou/mfcg/model/City.hx", lineNumber: 128, className: "com.watabou.mfcg.model.City", methodName: "new" });
	var h = com_watabou_mfcg_model_City.sizes.h;
	var group_h = h;
	var group_keys = Object.keys(h);
	var group_length = group_keys.length;
	var group_current = 0;
	while (group_current < group_length) {
		var group = group_h[group_keys[group_current++]];
		if (nPatches >= group.min && nPatches < group.max) {
			com_watabou_mfcg_model_City.nextSize = group.min + Math.random() * (group.max - group.min) | 0;
			break;
		}
	}
	this.citadelNeeded = bp.citadel;
	this.stadtburgNeeded = bp.inner;
	this.plazaNeeded = bp.plaza;
	this.templeNeeded = bp.temple;
	this.wallsNeeded = bp.walls;
	this.shantyNeeded = bp.shanty;
	this.riverNeeded = bp.river;
	this.coastNeeded = bp.coast;
	this.maxDocks = (Math.sqrt(nPatches / 2) | 0) + (this.riverNeeded ? 2 : 0);
	while (true) {
		try {
			haxe_Log.trace("-->> seed:" + com_watabou_utils_Random.seed + " size:" + nPatches, { fileName: "Source/com/watabou/mfcg/model/City.hx", lineNumber: 155, className: "com.watabou.mfcg.model.City", methodName: "new" });
			this.build();
			com_watabou_mfcg_model_City.instance = this;
		} catch (_g) {
			var _g1 = haxe_Exception.caught(_g);
			if (((_g1) instanceof openfl_errors_Error)) {
				var e = _g1;
				haxe_Log.trace("*** " + e.get_message(), { fileName: "Source/com/watabou/mfcg/model/City.hx", lineNumber: 160, className: "com.watabou.mfcg.model.City", methodName: "new" });
				com_watabou_mfcg_model_City.instance = null;
			} else {
				throw _g;
			}
		}
		if (!(com_watabou_mfcg_model_City.instance == null)) {
			break;
		}
	}
	bp.updateURL();
	com_watabou_mfcg_model_ModelDispatcher.newModel.dispatch(this);
};
$hxClasses["com.watabou.mfcg.model.City"] = com_watabou_mfcg_model_City;
com_watabou_mfcg_model_City.__name__ = "com.watabou.mfcg.model.City";
com_watabou_mfcg_model_City.prototype = {
	rerollName: function () {
		return com_watabou_mfcg_linguistics_Toponymy.cityName(this);
	}
	, setName: function (value, requested) {
		if (requested == null) {
			requested = false;
		}
		this.bp.name = this.name = value;
		this.bp.updateURL();
		com_watabou_mfcg_model_ModelDispatcher.titleChanged.dispatch(this.name);
	}
	, build: function () {
		this.streets = [];
		this.roads = [];
		this.walls = [];
		this.landmarks = [];
		this.north = 0.0;
		haxe_Log.trace("buildPatches " + com_watabou_utils_Stopwatch.measure($bind(this, this.buildPatches)), { fileName: "Source/com/watabou/mfcg/model/City.hx", lineNumber: 187, className: "com.watabou.mfcg.model.City", methodName: "build" });
		haxe_Log.trace("optimizeJunctions " + com_watabou_utils_Stopwatch.measure($bind(this, this.optimizeJunctions)), { fileName: "Source/com/watabou/mfcg/model/City.hx", lineNumber: 188, className: "com.watabou.mfcg.model.City", methodName: "build" });
		haxe_Log.trace("buildDomains " + com_watabou_utils_Stopwatch.measure($bind(this, this.buildDomains)), { fileName: "Source/com/watabou/mfcg/model/City.hx", lineNumber: 189, className: "com.watabou.mfcg.model.City", methodName: "build" });
		haxe_Log.trace("buildWalls " + com_watabou_utils_Stopwatch.measure($bind(this, this.buildWalls)), { fileName: "Source/com/watabou/mfcg/model/City.hx", lineNumber: 190, className: "com.watabou.mfcg.model.City", methodName: "build" });
		haxe_Log.trace("buildStreets " + com_watabou_utils_Stopwatch.measure($bind(this, this.buildStreets)), { fileName: "Source/com/watabou/mfcg/model/City.hx", lineNumber: 191, className: "com.watabou.mfcg.model.City", methodName: "build" });
		haxe_Log.trace("buildCanals " + com_watabou_utils_Stopwatch.measure($bind(this, this.buildCanals)), { fileName: "Source/com/watabou/mfcg/model/City.hx", lineNumber: 192, className: "com.watabou.mfcg.model.City", methodName: "build" });
		haxe_Log.trace("createWards " + com_watabou_utils_Stopwatch.measure($bind(this, this.createWards)), { fileName: "Source/com/watabou/mfcg/model/City.hx", lineNumber: 193, className: "com.watabou.mfcg.model.City", methodName: "build" });
		haxe_Log.trace("buildCityTowers " + com_watabou_utils_Stopwatch.measure($bind(this, this.buildCityTowers)), { fileName: "Source/com/watabou/mfcg/model/City.hx", lineNumber: 194, className: "com.watabou.mfcg.model.City", methodName: "build" });
		haxe_Log.trace("buildGeometry " + com_watabou_utils_Stopwatch.measure($bind(this, this.buildGeometry)), { fileName: "Source/com/watabou/mfcg/model/City.hx", lineNumber: 195, className: "com.watabou.mfcg.model.City", methodName: "build" });
		this.updateDimensions();
	}
	, buildPatches: function () {
		var _gthis = this;
		var points = [];
		var maxR = 0.0;
		var phase = 2 * Math.PI * ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647);
		com_watabou_utils_Random.save();
		if (this.plazaNeeded) {
			points.push(new openfl_geom_Point());
			var r1 = 8 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 * 12;
			var r2 = 8 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 * 12;
			maxR = Math.max(r1, r2);
			points.push(openfl_geom_Point.polar(r1, phase));
			points.push(openfl_geom_Point.polar(r2, phase + Math.PI / 2));
			points.push(openfl_geom_Point.polar(r1, phase + Math.PI));
			points.push(openfl_geom_Point.polar(r2, phase + Math.PI * 3 / 2));
		} else {
			var r = 0;
			var a = phase + Math.sqrt(0) * 5;
			points.push(openfl_geom_Point.polar(r, a));
			var r = 10 + (2 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647);
			var a = phase + Math.sqrt(1) * 5;
			points.push(openfl_geom_Point.polar(r, a));
			var r = 10 + 2 * (2 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647);
			var a = phase + Math.sqrt(2) * 5;
			points.push(openfl_geom_Point.polar(r, a));
			var r = 10 + 3 * (2 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647);
			var a = phase + Math.sqrt(3) * 5;
			points.push(openfl_geom_Point.polar(r, a));
			var r = 10 + 4 * (2 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647);
			var a = phase + Math.sqrt(4) * 5;
			points.push(openfl_geom_Point.polar(r, a));
		}
		com_watabou_utils_Random.restore();
		var _g = points.length;
		var _g1 = this.nPatches * 8;
		while (_g < _g1) {
			var i = _g++;
			var r = i == 0 ? 0 : 10 + i * (2 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647);
			var a = phase + Math.sqrt(i) * 5;
			points.push(openfl_geom_Point.polar(r, a));
			if (maxR < r) {
				maxR = r;
			}
		}
		var delanuator = new com_watabou_geom_Delaunator(points.concat([openfl_geom_Point.polar(2 * maxR, 0), openfl_geom_Point.polar(2 * maxR, Math.PI * 2 / 3), openfl_geom_Point.polar(2 * maxR, -Math.PI * 2 / 3)]));
		var map = delanuator.getVoronoi();
		var map1 = map;
		var _g2_map = map1;
		var _g2_keys = map1.keys();
		while (_g2_keys.hasNext()) {
			var key = _g2_keys.next();
			var _g3_value = _g2_map.get(key);
			var _g3_key = key;
			var point = _g3_key;
			var poly = _g3_value;
			var tooLarge = false;
			var _g = 0;
			while (_g < poly.length) {
				var p = poly[_g];
				++_g;
				if (p.get_length() > maxR) {
					tooLarge = true;
					break;
				}
			}
			if (tooLarge) {
				map.remove(point);
			}
		}
		this.patches = [];
		this.inner = [];
		var _g = [];
		var p = map.iterator();
		while (p.hasNext()) {
			var p1 = p.next();
			if (p1 != null) {
				_g.push(p1);
			}
		}
		this.dcel = new com_watabou_geom_DCEL(_g);
		var centers = new haxe_ds_ObjectMap();
		var _g = 0;
		var _g1 = this.dcel.faces;
		while (_g < _g1.length) {
			var f = _g1[_g];
			++_g;
			var patch = new com_watabou_mfcg_model_Patch(f.getPoly(), f);
			this.patches.push(patch);
			f.data = patch;
			var c = com_watabou_geom_polygons_PolyCore.centroid(patch.shape);
			centers.set(patch, c);
		}
		this.patches.sort(function (p1, p2) {
			var value = centers.h[p1.__id__].get_length() - centers.h[p2.__id__].get_length();
			if (value == 0) {
				return 0;
			} else if (value < 0) {
				return -1;
			} else {
				return 1;
			}
		});
		if (this.coastNeeded) {
			com_watabou_utils_Random.save();
			var a = this.riverNeeded ? ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 3 * 0.03 : ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 3 * 0.04 - 0.01;
			var b = 20 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 * 40;
			var f = this.bp.coastDir;
			if (isNaN(f)) {
				this.bp.coastDir = Math.floor((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 * 20) / 10;
			}
			var t = this.bp.coastDir * Math.PI;
			var cost = Math.cos(t);
			var sint = Math.sin(t);
			com_watabou_utils_Random.restore();
			var _g = 0;
			var _g1 = this.patches;
			while (_g < _g1.length) {
				var p = _g1[_g];
				++_g;
				var c = centers.h[p.__id__];
				var x2 = c.x * cost + c.y * sint;
				if (a * x2 * x2 + b + c.x * sint - c.y * cost < 0) {
					p.waterbody = true;
				}
			}
			this.headland = a < 0;
		}
		var count = 0;
		var _g = 0;
		var _g1 = this.patches;
		while (_g < _g1.length) {
			var patch = _g1[_g];
			++_g;
			if (!patch.waterbody) {
				patch.withinCity = true;
				patch.withinWalls = this.wallsNeeded;
				this.inner.push(patch);
				if (++count > this.nPatches) {
					break;
				}
			}
		}
		this.center = com_watabou_utils_ArrayExtender.min(this.inner[0].shape, function (p) {
			return p.get_length();
		});
		if (this.plazaNeeded) {
			new com_watabou_mfcg_model_wards_Market(this, this.plaza = this.inner[0]);
		}
		if (this.citadelNeeded) {
			if (this.stadtburgNeeded) {
				var _g = [];
				var _g1 = 0;
				var _g2 = this.inner;
				while (_g1 < _g2.length) {
					var v = _g2[_g1];
					++_g1;
					if ((function (p) {
						var _g = 0;
						var _g1 = [];
						var _g1_startEdge = p.face.halfEdge;
						var _g1_curEdge = _g1_startEdge;
						var _g1__hasNext = true;
						while (_g1__hasNext) {
							var result = _g1_curEdge;
							_g1_curEdge = _g1_curEdge.next;
							_g1__hasNext = _g1_curEdge != _g1_startEdge;
							var edge = result;
							_g1.push(edge);
						}
						var _g2 = _g1;
						while (_g < _g2.length) {
							var e = _g2[_g];
							++_g;
							var _g1 = 0;
							var _g3 = _gthis.patchesByVertex(e.origin);
							while (_g1 < _g3.length) {
								var p1 = _g3[_g1];
								++_g1;
								if (p1 != p && !p1.waterbody && !p1.withinCity) {
									return false;
								}
							}
						}
						return true;
					})(v)) {
						_g.push(v);
					}
				}
				var candidates = _g;
				HxOverrides.remove(candidates, this.plaza);
				var tmp;
				if (candidates.length > 0) {
					tmp = com_watabou_utils_ArrayExtender.random(candidates);
				} else {
					haxe_Log.trace("Unable to build an uraban castle!", { fileName: "Source/com/watabou/mfcg/model/City.hx", lineNumber: 335, className: "com.watabou.mfcg.model.City", methodName: "buildPatches" });
					var a = this.inner;
					tmp = this.citadel = a[a.length - 1];
				}
				this.citadel = tmp;
			} else {
				var a = this.inner;
				this.citadel = a[a.length - 1];
			}
			this.citadel.withinCity = true;
			this.citadel.withinWalls = true;
			HxOverrides.remove(this.inner, this.citadel);
		}
	}
	, fill: function () {
		var edge = [];
		var _g = 0;
		var _g1 = this.dcel.edges;
		while (_g < _g1.length) {
			var e = _g1[_g];
			++_g;
			if (e.twin == null) {
				com_watabou_utils_ArrayExtender.add(edge, e.face);
			}
		}
		var earth = [];
		var earthSeed = null;
		var water = [];
		var waterSeed = null;
		var _g = 0;
		while (_g < edge.length) {
			var f = edge[_g];
			++_g;
			var blob = com_watabou_geom_DCEL.floodFill(f, function (f) {
				return !f.data.waterbody;
			});
			if (blob.length > earth.length) {
				earth = blob;
				earthSeed = f;
			}
			var blob1 = com_watabou_geom_DCEL.floodFill(f, function (f) {
				return f.data.waterbody;
			});
			if (blob1.length > water.length) {
				water = blob1;
				waterSeed = f;
			}
		}
		var unfilled = com_watabou_utils_ArrayExtender.difference(com_watabou_utils_ArrayExtender.difference(this.dcel.faces, earth), water);
		while (unfilled.length > 0) {
			var waterState = [unfilled[0].data.waterbody];
			var blob = com_watabou_geom_DCEL.floodFill(unfilled[0], (function (waterState) {
				return function (f) {
					return f.data.waterbody == waterState[0];
				};
			})(waterState));
			var _g = 0;
			while (_g < blob.length) {
				var f = blob[_g];
				++_g;
				f.data.waterbody = !waterState[0];
			}
			if (waterState[0]) {
				earth = com_watabou_geom_DCEL.floodFill(earthSeed, (function () {
					return function (f) {
						return !f.data.waterbody;
					};
				})());
			} else {
				water = com_watabou_geom_DCEL.floodFill(waterSeed, (function () {
					return function (f) {
						return f.data.waterbody;
					};
				})());
			}
			unfilled = com_watabou_utils_ArrayExtender.difference(com_watabou_utils_ArrayExtender.difference(this.dcel.faces, earth), water);
		}
	}
	, optimizeJunctions: function () {
		while (true) {
			var modified = false;
			var _g = 0;
			var _g1 = this.dcel.faces;
			while (_g < _g1.length) {
				var f = _g1[_g];
				++_g;
				if (f.data == this.citadel) {
					continue;
				}
				var shape = f.data.shape;
				if (shape.length <= 4) {
					continue;
				}
				var p = com_watabou_geom_polygons_PolyCore.perimeter(shape);
				var t = Math.max(com_watabou_mfcg_model_CurtainWall.LTOWER_RADIUS * 3, p / shape.length / 3);
				var e = f.halfEdge;
				while (true) {
					var cit = this.citadel != null && this.citadel.shape.indexOf(e.origin.point) != -1 != (this.citadel.shape.indexOf(e.next.origin.point) != -1);
					if (e.twin != null && !cit && e.twin.face.data.shape.length > 4 && openfl_geom_Point.distance(e.origin.point, e.next.origin.point) < t) {
						var v = this.dcel.collapseEdge(e);
						var _g2 = 0;
						var _g3 = v.edges;
						while (_g2 < _g3.length) {
							var e1 = _g3[_g2];
							++_g2;
							e1.face.data.shape = e1.face.getPoly();
						}
						modified = true;
						break;
					}
					e = e.next;
					if (!(e != f.halfEdge)) {
						break;
					}
				}
			}
			if (!modified) {
				break;
			}
		}
		if (this.dcel.vertices.h[this.center.__id__] == null) {
			var minD = Infinity;
			var p = this.dcel.vertices.keys();
			while (p.hasNext()) {
				var p1 = p.next();
				var d = p1.get_length();
				if (minD > d) {
					minD = d;
					this.center = p1;
				}
			}
		}
	}
	, buildWalls: function () {
		com_watabou_utils_Random.save();
		var reserved = this.waterEdge;
		if (this.citadel != null) {
			reserved = reserved.concat(this.citadel.shape);
		}
		this.border = new com_watabou_mfcg_model_CurtainWall(this.wallsNeeded, this, this.inner, reserved);
		if (this.wallsNeeded) {
			this.wall = this.border;
			this.walls.push(this.wall);
		}
		this.gates = this.border.gates;
		if (this.citadel != null) {
			var castle = new com_watabou_mfcg_model_wards_Castle(this, this.citadel);
			castle.wall.buildTowers();
			this.walls.push(castle.wall);
			this.gates = this.gates.concat(castle.wall.gates);
		}
		com_watabou_utils_Random.restore();
	}
	, patchesByVertex: function (v) {
		var _g = [];
		var _g1 = 0;
		var _g2 = v.edges;
		while (_g1 < _g2.length) {
			var e = _g2[_g1];
			++_g1;
			_g.push(e.face.data);
		}
		return _g;
	}
	, buildDomains: function () {
		this.dcel.faces.reverse();
		this.horizonE = com_watabou_geom_DCEL.circumference(null, com_watabou_utils_SetUtils.fromArray(this.dcel.faces));
		if (this.horizonE.length < 6) {
			throw new openfl_errors_Error("Failed to build the horizon");
		}
		com_watabou_geom_EdgeChain.assignData(this.horizonE, com_watabou_mfcg_model_PatchEdge.HORIZON);
		this.horizon = com_watabou_geom_EdgeChain.toPoly(this.horizonE);
		if (this.coastNeeded) {
			var water = new haxe_ds_ObjectMap();
			var earth = new haxe_ds_ObjectMap();
			var _g = 0;
			var _g1 = this.dcel.faces;
			while (_g < _g1.length) {
				var f = _g1[_g];
				++_g;
				if (f.data.waterbody) {
					water.set(f, true);
				} else {
					earth.set(f, true);
				}
			}
			var islands = [];
			while (!com_watabou_utils_SetUtils.isEmpty(earth)) {
				var island = com_watabou_geom_DCEL.floodFill(com_watabou_utils_SetUtils.pick(earth), function (f) {
					return !f.data.waterbody;
				});
				com_watabou_utils_SetUtils.removeArr(earth, island);
				islands.push(island);
			}
			earth = com_watabou_utils_SetUtils.fromArray(com_watabou_utils_ArrayExtender.max(islands, function (island) {
				return island.length;
			}));
			this.earthEdgeE = com_watabou_geom_DCEL.circumference(null, earth);
			this.earthEdge = com_watabou_geom_EdgeChain.toPoly(this.earthEdgeE);
			this.waterEdgeE = com_watabou_geom_DCEL.circumference(null, water);
			this.waterEdge = com_watabou_geom_EdgeChain.toPoly(this.waterEdgeE);
			com_watabou_geom_polygons_PolyCore.set(this.waterEdge, com_watabou_mfcg_utils_PolyUtils.smooth(this.waterEdge, null, Math.floor(1 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 * 3)));
			var pos = 0;
			while (this.earthEdgeE[pos].twin != null) pos = (pos + 1) % this.earthEdgeE.length;
			while (this.earthEdgeE[pos].twin == null) pos = (pos + 1) % this.earthEdgeE.length;
			this.shore = [];
			this.shoreE = [];
			while (true) {
				var e = this.earthEdgeE[pos];
				this.shoreE.push(e);
				this.shore.push(e.origin.point);
				pos = (pos + 1) % this.earthEdgeE.length;
				if (!(this.earthEdgeE[pos].twin != null)) {
					break;
				}
			}
			com_watabou_geom_EdgeChain.assignData(this.shoreE, com_watabou_mfcg_model_PatchEdge.COAST);
		} else {
			this.earthEdgeE = this.horizonE;
			this.earthEdge = this.horizon;
			this.waterEdgeE = [];
			this.waterEdge = [];
			this.shoreE = [];
			this.shore = [];
		}
	}
	, buildStreets: function () {
		var _gthis = this;
		var _g = [];
		var _g1 = 0;
		var _g2 = this.patches;
		while (_g1 < _g2.length) {
			var p = _g2[_g1];
			++_g1;
			if (p.withinCity) {
				_g.push(p);
			}
		}
		var innerTopology = new com_watabou_mfcg_model_Topology(_g);
		var _g = [];
		var _g1 = 0;
		var _g2 = this.patches;
		while (_g1 < _g2.length) {
			var p = _g2[_g1];
			++_g1;
			if (!p.withinCity && !p.waterbody) {
				_g.push(p);
			}
		}
		var outerTopology = new com_watabou_mfcg_model_Topology(_g);
		var towers = [];
		if (this.wall != null) {
			com_watabou_utils_ArrayExtender.addAll(towers, com_watabou_geom_EdgeChain.vertices(this.wall.edges));
		}
		if (this.citadel != null) {
			com_watabou_utils_ArrayExtender.addAll(towers, com_watabou_geom_EdgeChain.vertices((js_Boot.__cast(this.citadel.ward, com_watabou_mfcg_model_wards_Castle)).wall.edges));
		}
		if (towers.length > 0) {
			com_watabou_utils_ArrayExtender.removeAll(towers, this.gates);
			innerTopology.excludePoints(towers);
			outerTopology.excludePoints(towers);
		}
		var earthEdge = com_watabou_utils_ArrayExtender.difference(this.earthEdgeE, this.shoreE);
		var _g = [];
		var _g1 = 0;
		var _g2 = earthEdge;
		while (_g1 < _g2.length) {
			var v = _g2[_g1];
			++_g1;
			if (outerTopology.pt2node.h.__keys__[v.origin.__id__] != null) {
				_g.push(v);
			}
		}
		earthEdge = _g;
		var _g = 0;
		var _g1 = this.gates;
		while (_g < _g1.length) {
			var gate = [_g1[_g]];
			++_g;
			var end = this.plaza != null ? com_watabou_utils_ArrayExtender.min(this.plaza.shape, (function (gate) {
				return function (v) {
					return openfl_geom_Point.distance(v, gate[0].point);
				};
			})(gate)) : this.center;
			var streetVertices = innerTopology.buildPath(gate[0], this.dcel.vertices.h[end.__id__]);
			if (streetVertices != null) {
				var street = this.dcel.vertices2chain(streetVertices);
				this.streets.push(street);
				if (this.border.gates.indexOf(gate[0]) != -1) {
					var bestRoad = null;
					if (outerTopology.pt2node.h.__keys__[gate[0].__id__] != null) {
						earthEdge = com_watabou_utils_ArrayExtender.sortBy(earthEdge, (function (gate) {
							return function (v) {
								var p1 = gate[0].point;
								var p2 = v.origin.point;
								return -(p1.x * p2.x + p1.y * p2.y) / v.origin.point.get_length();
							};
						})(gate));
						var _g2 = 0;
						while (_g2 < earthEdge.length) {
							var v = earthEdge[_g2];
							++_g2;
							bestRoad = outerTopology.buildPath(v.origin, gate[0]);
							if (bestRoad != null) {
								break;
							}
						}
					}
					var roadVertices = bestRoad;
					if (roadVertices != null) {
						var road = this.dcel.vertices2chain(roadVertices);
						outerTopology.excludePolygon(road);
						this.roads.push(road);
					} else if (this.wall != null) {
						var _g3 = [];
						var _g4 = 0;
						var _g5 = this.patchesByVertex(gate[0]);
						while (_g4 < _g5.length) {
							var v1 = _g5[_g4];
							++_g4;
							if (!v1.withinWalls && v1.bordersInside(_gthis.shoreE)) {
								_g3.push(v1);
							}
						}
						var outerDocks = _g3;
						var _g6 = 0;
						while (_g6 < outerDocks.length) {
							var patch = outerDocks[_g6];
							++_g6;
							patch.landing = true;
							patch.withinCity = true;
							new com_watabou_mfcg_model_wards_Alleys(this, patch);
							this.maxDocks--;
						}
					}
				}
			} else {
				haxe_Log.trace("Unable to build a street!", { fileName: "Source/com/watabou/mfcg/model/City.hx", lineNumber: 598, className: "com.watabou.mfcg.model.City", methodName: "buildStreets" });
			}
		}
		this.tidyUpRoads();
		var reserved;
		if (this.wallsNeeded) {
			var _g = [];
			var _g1 = 0;
			var _g2 = this.gates;
			while (_g1 < _g2.length) {
				var g = _g2[_g1];
				++_g1;
				_g.push(g.point);
			}
			reserved = _g;
		} else {
			reserved = null;
		}
		var _g = 0;
		var _g1 = this.arteries;
		while (_g < _g1.length) {
			var a = _g1[_g];
			++_g;
			com_watabou_geom_EdgeChain.assignData(a, com_watabou_mfcg_model_PatchEdge.ROAD);
			var poly = com_watabou_geom_EdgeChain.toPoly(a);
			com_watabou_geom_polygons_PolyCore.set(poly, com_watabou_mfcg_utils_PolyUtils.smoothOpen(poly, reserved, 2));
		}
	}
	, tidyUpRoads: function () {
		var edges = [];
		var _g = 0;
		var _g1 = this.streets;
		while (_g < _g1.length) {
			var street = _g1[_g];
			++_g;
			com_watabou_utils_ArrayExtender.addAll(edges, street);
		}
		var _g = 0;
		var _g1 = this.roads;
		while (_g < _g1.length) {
			var road = _g1[_g];
			++_g;
			com_watabou_utils_ArrayExtender.addAll(edges, road);
		}
		this.arteries = [];
		while (edges.length > 0) {
			var edge = edges.pop();
			var attached = false;
			var _g = 0;
			var _g1 = this.arteries;
			while (_g < _g1.length) {
				var a = _g1[_g];
				++_g;
				if (a[0].origin == edge.next.origin) {
					a.unshift(edge);
					attached = true;
					break;
				} else if (a[a.length - 1].next.origin == edge.origin) {
					a.push(edge);
					attached = true;
					break;
				}
			}
			if (!attached) {
				this.arteries.push([edge]);
			}
		}
	}
	, buildCanals: function () {
		com_watabou_utils_Random.save();
		this.canals = this.riverNeeded ? [com_watabou_mfcg_model_Canal.createRiver(this)] : [];
		com_watabou_utils_Random.restore();
	}
	, addHarbour: function (p) {
		var _g = 0;
		var neighbours = [];
		var e = p.face.halfEdge;
		while (true) {
			if (e.twin != null) {
				neighbours.push(e.twin.face.data);
			}
			e = e.next;
			if (!(e != p.face.halfEdge)) {
				break;
			}
		}
		var _g1 = neighbours;
		while (_g < _g1.length) {
			var n = _g1[_g];
			++_g;
			if (n.waterbody && n.ward == null) {
				new com_watabou_mfcg_model_wards_Harbour(this, n);
			}
		}
	}
	, createWards: function () {
		if (this.bp.greens) {
			var nParks = 0;
			if (this.citadel != null) {
				var castle = this.citadel.ward;
				var approach = this.patchesByVertex(castle.wall.gates[0]);
				var tmp;
				if (approach.length == 3) {
					var chance = 1 - 2 / (this.nPatches - 1);
					if (chance == null) {
						chance = 0.5;
					}
					tmp = (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance;
				} else {
					tmp = false;
				}
				if (tmp) {
					var _g = 0;
					while (_g < approach.length) {
						var p = approach[_g];
						++_g;
						if (p.ward == null) {
							new com_watabou_mfcg_model_wards_Park(this, p);
							++nParks;
						}
					}
				}
			}
			var f = (this.nPatches - 10) / 20;
			var chance = f - (f | 0);
			if (chance == null) {
				chance = 0.5;
			}
			var nParks1 = (f | 0) + ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance ? 1 : 0) - nParks;
			var _g = 0;
			var _g1 = nParks1;
			while (_g < _g1) {
				var i = _g++;
				while (true) {
					var patch = com_watabou_utils_ArrayExtender.random(this.inner);
					if (patch.ward == null) {
						new com_watabou_mfcg_model_wards_Park(this, patch);
						break;
					}
				}
			}
		}
		if (this.shoreE.length > 0 && this.maxDocks > 0) {
			var _g = 0;
			var _g1 = this.inner;
			while (_g < _g1.length) {
				var p = _g1[_g];
				++_g;
				if (p.bordersInside(this.shoreE)) {
					p.landing = true;
					if (--this.maxDocks <= 0) {
						break;
					}
				}
			}
		}
		if (this.templeNeeded) {
			var patch = com_watabou_utils_ArrayExtender.min(this.inner, function (p) {
				if (p.ward == null) {
					return com_watabou_geom_polygons_PolyCore.center(p.shape).get_length();
				} else {
					return Infinity;
				}
			});
			new com_watabou_mfcg_model_wards_Cathedral(this, patch);
		}
		var nMarkets = 0;
		var _g = 0;
		var _g1 = nMarkets;
		while (_g < _g1) {
			var _ = _g++;
			while (true) {
				var patch = com_watabou_utils_ArrayExtender.random(this.inner);
				if (patch.ward == null) {
					new com_watabou_mfcg_model_wards_Market(this, patch);
					break;
				}
			}
		}
		var _g = 0;
		var _g1 = this.inner;
		while (_g < _g1.length) {
			var patch = _g1[_g];
			++_g;
			if (patch.ward == null) {
				new com_watabou_mfcg_model_wards_Alleys(this, patch);
			}
		}
		if (this.wall != null) {
			var _g = 0;
			var _g1 = this.wall.gates;
			while (_g < _g1.length) {
				var gate = _g1[_g];
				++_g;
				var chance = 1 / (this.nPatches - 5);
				if (chance == null) {
					chance = 0.5;
				}
				if (!((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance)) {
					var _g2 = 0;
					var _g3 = this.patchesByVertex(gate);
					while (_g2 < _g3.length) {
						var patch = _g3[_g2];
						++_g2;
						if (patch.ward == null) {
							patch.withinCity = true;
							if (patch.bordersInside(this.shoreE) && this.maxDocks-- > 0) {
								patch.landing = true;
							}
							new com_watabou_mfcg_model_wards_Alleys(this, patch);
						}
					}
				}
			}
		}
		if (this.shantyNeeded) {
			this.buildShantyTowns();
		}
		var _g = 0;
		var _g1 = com_watabou_geom_EdgeChain.vertices(this.shoreE);
		while (_g < _g1.length) {
			var p = _g1[_g];
			++_g;
			var _g2 = 0;
			var _g3 = this.patchesByVertex(p);
			while (_g2 < _g3.length) {
				var n = _g3[_g2];
				++_g2;
				if (n.withinCity && !n.landing) {
					var e = n.face.halfEdge;
					while (e.next.origin != p) e = e.next;
					if (e.twin.face.data.landing && e.next.twin.face.data.landing) {
						n.landing = true;
						break;
					}
				}
			}
		}
		var _g = 0;
		var _g1 = this.patches;
		while (_g < _g1.length) {
			var p = _g1[_g];
			++_g;
			if (p.landing) {
				this.addHarbour(p);
			}
		}
		this.buildFarms();
	}
	, updateDimensions: function () {
		var _gthis = this;
		this.minx = 0;
		this.maxx = 0;
		this.miny = 0;
		this.maxy = 0;
		var measure = function (shape) {
			var _g = 0;
			while (_g < shape.length) {
				var v = shape[_g];
				++_g;
				if (v.x < _gthis.minx) {
					_gthis.minx = v.x;
				} else if (v.x > _gthis.maxx) {
					_gthis.maxx = v.x;
				}
				if (v.y < _gthis.miny) {
					_gthis.miny = v.y;
				} else if (v.y > _gthis.maxy) {
					_gthis.maxy = v.y;
				}
			}
		};
		var _g = 0;
		var _g1 = this.districts;
		while (_g < _g1.length) {
			var district = _g1[_g];
			++_g;
			var _g2 = 0;
			var _g3 = district.groups;
			while (_g2 < _g3.length) {
				var group = _g3[_g2];
				++_g2;
				var _g4 = 0;
				var _g5 = group.blocks;
				while (_g4 < _g5.length) {
					var block = _g5[_g4];
					++_g4;
					measure(block.shape);
				}
			}
		}
		if (this.citadel != null) {
			measure((js_Boot.__cast(this.citadel.ward, com_watabou_mfcg_model_wards_Castle)).wall.shape);
		}
	}
	, buildFarms: function () {
		var a1 = ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 3 * 2;
		var a2 = ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 3;
		var p1 = (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 * Math.PI * 2;
		var p2 = (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 * Math.PI * 2;
		var innerRadius = 0.0;
		var _g = 0;
		var _g1 = this.inner;
		while (_g < _g1.length) {
			var patch = _g1[_g];
			++_g;
			var _g2 = 0;
			var _g3 = patch.shape;
			while (_g2 < _g3.length) {
				var v = _g3[_g2];
				++_g2;
				innerRadius = Math.max(innerRadius, openfl_geom_Point.distance(v, this.center));
			}
		}
		var _g = 0;
		var _g1 = this.patches;
		while (_g < _g1.length) {
			var patch = _g1[_g];
			++_g;
			if (patch.ward == null) {
				if (patch.waterbody || patch.bordersInside(this.shoreE)) {
					new com_watabou_mfcg_model_wards_Ward(this, patch);
				} else {
					var c = com_watabou_geom_polygons_PolyCore.center(patch.shape).subtract(this.center);
					var a = Math.atan2(c.y, c.x);
					var v = a1 * Math.sin(a + p1) + a2 * Math.sin(a * 2 + p2);
					if (this.bp.farms && c.get_length() < (v + 1) * innerRadius) {
						new com_watabou_mfcg_model_wards_Farm(this, patch);
					} else {
						new com_watabou_mfcg_model_wards_Ward(this, patch);
					}
				}
			}
		}
	}
	, buildShantyTowns: function () {
		var _gthis = this;
		var lands = [];
		var weights = [];
		var distance2attractors = function (c) {
			var d = openfl_geom_Point.distance(c, _gthis.center) * 3;
			var _g = 0;
			var _g1 = _gthis.roads;
			while (_g < _g1.length) {
				var road = _g1[_g];
				++_g;
				var _g2 = 0;
				while (_g2 < road.length) {
					var p = road[_g2];
					++_g2;
					d = Math.min(d, openfl_geom_Point.distance(p.origin.point, c) * 2);
				}
			}
			var _g = 0;
			var _g1 = _gthis.shoreE;
			while (_g < _g1.length) {
				var e = _g1[_g];
				++_g;
				d = Math.min(d, openfl_geom_Point.distance(e.origin.point, c));
			}
			var _g = 0;
			var _g1 = _gthis.canals;
			while (_g < _g1.length) {
				var canal = _g1[_g];
				++_g;
				var _g2 = 0;
				var _g3 = canal.course;
				while (_g2 < _g3.length) {
					var p = _g3[_g2];
					++_g2;
					d = Math.min(d, openfl_geom_Point.distance(p.origin.point, c));
				}
			}
			return d * d;
		};
		var addNeighbours = function (patch) {
			var _g = 0;
			var neighbours = [];
			var e = patch.face.halfEdge;
			while (true) {
				if (e.twin != null) {
					neighbours.push(e.twin.face.data);
				}
				e = e.next;
				if (!(e != patch.face.halfEdge)) {
					break;
				}
			}
			var _g1 = neighbours;
			while (_g < _g1.length) {
				var n = _g1[_g];
				++_g;
				if (n != null) {
					if (!n.withinCity && !n.waterbody && !n.bordersInside(_gthis.horizonE)) {
						var neighbours = [];
						var e = n.face.halfEdge;
						while (true) {
							if (e.twin != null) {
								neighbours.push(e.twin.face.data);
							}
							e = e.next;
							if (!(e != n.face.halfEdge)) {
								break;
							}
						}
						var c = com_watabou_utils_ArrayExtender.count(neighbours, function (p) {
							return p.withinCity;
						});
						if (c > 1 && com_watabou_utils_ArrayExtender.add(lands, n)) {
							weights.push(c * c / distance2attractors(com_watabou_geom_polygons_PolyCore.center(n.shape)));
						}
					}
				}
			}
		};
		var _g = 0;
		var _g1 = this.patches;
		while (_g < _g1.length) {
			var patch = _g1[_g];
			++_g;
			if (patch.withinCity) {
				addNeighbours(patch);
			}
		}
		var f = (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647;
		var count = this.nPatches * (1 + f * f * f) * 0.5;
		while (count > 0 && lands.length > 0) {
			var _g = [];
			var _g1 = 0;
			var _g2 = weights.length;
			while (_g1 < _g2) {
				var i = _g1++;
				_g.push(i);
			}
			var i1 = com_watabou_utils_ArrayExtender.weighted(_g, weights);
			var p = lands[i1];
			p.withinCity = true;
			if (this.maxDocks > 0 && p.bordersInside(this.shoreE)) {
				p.landing = true;
				this.maxDocks--;
			}
			new com_watabou_mfcg_model_wards_Alleys(this, p);
			weights.splice(i1, 1);
			HxOverrides.remove(lands, p);
			--count;
			addNeighbours(p);
		}
	}
	, buildCityTowers: function () {
		if (this.wall != null) {
			var _g = 0;
			var _g1 = this.wall.edges.length;
			while (_g < _g1) {
				var i = _g++;
				var edge = this.wall.edges[i];
				if (edge.data == com_watabou_mfcg_model_PatchEdge.COAST || edge.twin.face.data == this.citadel) {
					this.wall.segments[i] = false;
				}
			}
			this.wall.buildTowers();
			if (this.citadel != null) {
				var _g = 0;
				var _g1 = (js_Boot.__cast(this.citadel.ward, com_watabou_mfcg_model_wards_Castle)).wall.towers;
				while (_g < _g1.length) {
					var t = _g1[_g];
					++_g;
					HxOverrides.remove(this.wall.towers, t);
				}
			}
		}
	}
	, getTideline: function () {
		if (this.tideLine != null) {
			return this.tideLine;
		}
		var exclude = [];
		var wasLanding = false;
		var _g = 0;
		var _g1 = this.earthEdgeE;
		while (_g < _g1.length) {
			var e = _g1[_g];
			++_g;
			if (e.twin == null) {
				exclude.push(e.origin.point);
			} else {
				var patch = e.face.data;
				var isLanding = false;
				var excluded = false;
				if (patch.landing) {
					isLanding = true;
				} else if (patch.withinCity && !com_watabou_geom_polygons_PolyAccess.isConvexVertexi(this.waterEdge, this.waterEdge.indexOf(e.origin.point))) {
					excluded = true;
				}
				if (wasLanding || isLanding) {
					excluded = true;
				}
				wasLanding = isLanding;
				if (excluded) {
					exclude.push(e.origin.point);
				}
			}
		}
		this.tideLine = com_watabou_geom_Chaikin.render(this.earthEdge, true, 3, exclude);
		return this.tideLine;
	}
	, buildGeometry: function () {
		var _g = 0;
		var _g1 = this.canals;
		while (_g < _g1.length) {
			var canal = _g1[_g];
			++_g;
			canal.updateState();
		}
		this.name = this.bp.name != null ? this.bp.name : this.rerollName();
		var db = new com_watabou_mfcg_model_DistrictBuilder(this);
		db.build();
		this.districts = db.districts;
		var _g = 0;
		var _g1 = this.patches;
		while (_g < _g1.length) {
			var patch = _g1[_g];
			++_g;
			patch.ward.createGeometry();
		}
	}
	, rerollDistricts: function () {
		var _g = 0;
		var _g1 = this.districts;
		while (_g < _g1.length) {
			var d = _g1[_g];
			++_g;
			d.name = null;
		}
		new com_watabou_mfcg_linguistics_DistrictNames(this, this.districts).generate();
		com_watabou_mfcg_model_ModelDispatcher.districtsChanged.dispatch();
	}
	, updateGeometry: function (patches) {
		var districts2Update = [];
		var alleys2Update = [];
		var _g = 0;
		while (_g < patches.length) {
			var patch = patches[_g];
			++_g;
			if (((patch.ward) instanceof com_watabou_mfcg_model_wards_Alleys)) {
				com_watabou_utils_ArrayExtender.add(alleys2Update, patch.ward.group.core);
			} else {
				patch.ward.createGeometry();
			}
			if (patch.district != null) {
				com_watabou_utils_ArrayExtender.add(districts2Update, patch.district);
			}
		}
		var _g = 0;
		while (_g < alleys2Update.length) {
			var patch = alleys2Update[_g];
			++_g;
			patch.ward.createGeometry();
		}
		var _g = 0;
		while (_g < districts2Update.length) {
			var district = districts2Update[_g];
			++_g;
			district.updateGeometry();
		}
		com_watabou_mfcg_model_ModelDispatcher.geometryChanged.dispatch(patches.length == 1 ? patches[0] : null);
	}
	, updateLots: function () {
		var _g = 0;
		var _g1 = this.patches;
		while (_g < _g1.length) {
			var patch = _g1[_g];
			++_g;
			if (((patch.ward) instanceof com_watabou_mfcg_model_wards_Alleys)) {
				patch.ward.createGeometry();
			}
		}
		com_watabou_mfcg_model_ModelDispatcher.geometryChanged.dispatch(null);
	}
	, addLandmark: function (x, y) {
		var lm = new com_watabou_mfcg_model_Landmark(this, new openfl_geom_Point(x, y));
		this.landmarks.push(lm);
		com_watabou_mfcg_model_ModelDispatcher.landmarksChanged.dispatch();
		return lm;
	}
	, updateLandmarks: function () {
		var _g = 0;
		var _g1 = this.landmarks;
		while (_g < _g1.length) {
			var l = _g1[_g];
			++_g;
			l.update();
		}
	}
	, addLandmarks: function (list) {
		var lots = [];
		var _g = 0;
		var _g1 = this.districts;
		while (_g < _g1.length) {
			var district = _g1[_g];
			++_g;
			var _g2 = 0;
			var _g3 = district.groups;
			while (_g2 < _g3.length) {
				var group = _g3[_g2];
				++_g2;
				var _g4 = 0;
				var _g5 = group.blocks;
				while (_g4 < _g5.length) {
					var block = _g5[_g4];
					++_g4;
					var b = block.lots;
					var _g6 = 0;
					while (_g6 < b.length) {
						var e = b[_g6];
						++_g6;
						lots.push(e);
					}
				}
			}
		}
		var _g = 0;
		while (_g < list.length) {
			var name = list[_g];
			++_g;
			var lot = com_watabou_utils_ArrayExtender.random(lots);
			var pos = com_watabou_geom_polygons_PolyCore.center(lot);
			var lm = new com_watabou_mfcg_model_Landmark(this, pos, name);
			this.landmarks.push(lm);
			HxOverrides.remove(lots, lot);
		}
		com_watabou_mfcg_model_ModelDispatcher.landmarksChanged.dispatch();
	}
	, removeLandmark: function (lm) {
		HxOverrides.remove(this.landmarks, lm);
		com_watabou_mfcg_model_ModelDispatcher.landmarksChanged.dispatch();
	}
	, removeLandmarks: function () {
		this.landmarks = [];
		com_watabou_mfcg_model_ModelDispatcher.landmarksChanged.dispatch();
	}
	, countBuildings: function () {
		var count = 0;
		var _g = 0;
		var _g1 = this.districts;
		while (_g < _g1.length) {
			var district = _g1[_g];
			++_g;
			var _g2 = 0;
			var _g3 = district.groups;
			while (_g2 < _g3.length) {
				var group = _g3[_g2];
				++_g2;
				var _g4 = 0;
				var _g5 = group.blocks;
				while (_g4 < _g5.length) {
					var block = _g5[_g4];
					++_g4;
					count += block.lots.length;
				}
			}
		}
		return count;
	}
	, getTrees: function () {
		var trees = [];
		var _g = 0;
		var _g1 = this.patches;
		while (_g < _g1.length) {
			var patch = _g1[_g];
			++_g;
			var wardTrees = patch.ward.spawnTrees();
			if (wardTrees != null) {
				var _g2 = 0;
				while (_g2 < wardTrees.length) {
					var e = wardTrees[_g2];
					++_g2;
					trees.push(e);
				}
			}
		}
		return trees;
	}
	, getNeighbour: function (patch, v) {
		var e = patch.face.halfEdge;
		while (true) {
			if (e.origin == v) {
				if (e.twin != null) {
					return e.twin.face.data;
				} else {
					return null;
				}
			}
			e = e.next;
			if (!(e != patch.face.halfEdge)) {
				break;
			}
		}
		return null;
	}
	, getPatch: function (point) {
		var patch = null;
		var _g = 0;
		var _g1 = this.patches;
		while (_g < _g1.length) {
			var p = _g1[_g];
			++_g;
			if (com_watabou_geom_polygons_PolyBounds.containsPoint(p.shape, point)) {
				patch = p;
				break;
			}
		}
		return patch;
	}
	, getNeighbours: function (patch) {
		var neighbours = [];
		var e = patch.face.halfEdge;
		while (true) {
			if (e.twin != null) {
				neighbours.push(e.twin.face.data);
			}
			e = e.next;
			if (!(e != patch.face.halfEdge)) {
				break;
			}
		}
		return neighbours;
	}
	, getDetails: function (rect) {
		var weight = 0;
		var _g = 0;
		var _g1 = this.districts;
		while (_g < _g1.length) {
			var district = _g1[_g];
			++_g;
			var _g2 = 0;
			var _g3 = district.groups;
			while (_g2 < _g3.length) {
				var group = _g3[_g2];
				++_g2;
				var _g4 = 0;
				var _g5 = group.blocks;
				while (_g4 < _g5.length) {
					var block = _g5[_g4];
					++_g4;
					var _g6 = 0;
					var _g7 = block.shape;
					while (_g6 < _g7.length) {
						var p = _g7[_g6];
						++_g6;
						if (rect.containsPoint(p)) {
							++weight;
							break;
						}
					}
				}
			}
		}
		var _g = 0;
		var _g1 = this.walls;
		while (_g < _g1.length) {
			var wall = _g1[_g];
			++_g;
			var _g2 = 0;
			var _g3 = wall.shape;
			while (_g2 < _g3.length) {
				var p = _g3[_g2];
				++_g2;
				if (rect.containsPoint(p)) {
					++weight;
				}
			}
		}
		return weight;
	}
	, splitEdge: function (e) {
		var newEdge = this.dcel.splitEdge(e);
		e.face.data.shape = e.face.getPoly();
		e.twin.face.data.shape = e.twin.face.getPoly();
		return newEdge;
	}
	, getFMGParams: function () {
		return {
			size: this.nPatches, seed: this.bp.seed, name: this.name, coast: this.shoreE.length > 0 ? 1 : 0, port: com_watabou_utils_ArrayExtender.some(this.patches, function (p) {
				return p.landing;
			}) ? 1 : 0, river: this.canals.length > 0 ? 1 : 0, sea: this.shoreE.length > 0 ? this.bp.coastDir : 0
		};
	}
	, __class__: com_watabou_mfcg_model_City
};
var com_watabou_mfcg_model_CurtainWall = function (real, model, patches, reserved) {
	this.watergates = new haxe_ds_ObjectMap();
	this.real = true;
	this.patches = patches;
	if (patches.length == 1) {
		var _g = [];
		var _g1_startEdge = patches[0].face.halfEdge;
		var _g1_curEdge = _g1_startEdge;
		var _g1__hasNext = true;
		while (_g1__hasNext) {
			var result = _g1_curEdge;
			_g1_curEdge = _g1_curEdge.next;
			_g1__hasNext = _g1_curEdge != _g1_startEdge;
			var edge = result;
			_g.push(edge);
		}
		this.edges = _g;
		this.shape = patches[0].shape;
	} else {
		var set = new haxe_ds_ObjectMap();
		var _g = 0;
		while (_g < patches.length) {
			var p = patches[_g];
			++_g;
			set.set(p.face, true);
		}
		this.edges = com_watabou_geom_DCEL.circumference(null, set);
		this.shape = com_watabou_geom_EdgeChain.toPoly(this.edges);
	}
	if (real) {
		com_watabou_geom_EdgeChain.assignData(this.edges, com_watabou_mfcg_model_PatchEdge.WALL, false);
		if (patches.length > 1) {
			com_watabou_geom_polygons_PolyCore.set(this.shape, com_watabou_mfcg_utils_PolyUtils.smooth(this.shape, reserved, 2));
		}
	}
	this.length = this.shape.length;
	if (patches.length == 1) {
		this.buildCastleGate(model, reserved);
	} else {
		this.buildCityGates(real, model, reserved);
	}
	var _g = [];
	var _g1 = 0;
	var _g2 = this.shape;
	while (_g1 < _g2.length) {
		var v = _g2[_g1];
		++_g1;
		_g.push(true);
	}
	this.segments = _g;
};
$hxClasses["com.watabou.mfcg.model.CurtainWall"] = com_watabou_mfcg_model_CurtainWall;
com_watabou_mfcg_model_CurtainWall.__name__ = "com.watabou.mfcg.model.CurtainWall";
com_watabou_mfcg_model_CurtainWall.prototype = {
	buildCityGates: function (real, model, reserved) {
		this.gates = [];
		var _g = [];
		var _g1 = 0;
		var _g2 = this.edges;
		while (_g1 < _g2.length) {
			var e = _g2[_g1];
			++_g1;
			_g.push(reserved.indexOf(e.origin.point) != -1 || com_watabou_utils_ArrayExtender.intersect(model.patchesByVertex(e.origin), this.patches).length < 2 ? 0.0 : 1.0);
		}
		var weights = _g;
		if (com_watabou_utils_ArrayExtender.sum(weights) == 0) {
			haxe_Log.trace("" + this.length + " vertices of " + this.patches.length + " patches, " + reserved.length + " are reserved.", { fileName: "Source/com/watabou/mfcg/model/CurtainWall.hx", lineNumber: 83, className: "com.watabou.mfcg.model.CurtainWall", methodName: "buildCityGates" });
			throw new openfl_errors_Error("No valid vertices to create gates!");
		}
		var maxGates = model.bp.gates > -1 ? model.bp.gates : model.bp.hub ? this.shape.length : 2 + (this.patches.length / 12 * (model.shoreE.length > 0 ? 0.75 : 1.0) | 0);
		while (this.gates.length < maxGates && com_watabou_utils_ArrayExtender.sum(weights) > 0) {
			var _g = [];
			var _g1 = 0;
			var _g2 = weights.length;
			while (_g1 < _g2) {
				var i = _g1++;
				_g.push(i);
			}
			var index = com_watabou_utils_ArrayExtender.weighted(_g, weights);
			var gate = [this.edges[index].origin];
			this.gates.push(gate[0]);
			if (real) {
				var outerWards = com_watabou_utils_ArrayExtender.difference(model.patchesByVertex(gate[0]), this.patches);
				if (outerWards.length == 1) {
					var outer = outerWards[0];
					var points = com_watabou_utils_ArrayExtender.difference(outer.shape, reserved);
					var poly = this.shape;
					var i1 = poly.indexOf(gate[0].point);
					var prev;
					if (i1 != -1) {
						var len = poly.length;
						prev = poly[(i1 + len - 1) % len];
					} else {
						prev = null;
					}
					var poly1 = this.shape;
					var i2 = poly1.indexOf(gate[0].point);
					var next = i2 != -1 ? poly1[(i2 + 1) % poly1.length] : null;
					com_watabou_utils_ArrayExtender.removeAll(points, this.shape);
					if (points.length > 0) {
						var out = [gate[0].point.subtract(com_watabou_geom_GeomUtils.lerp(prev, next))];
						var farthest = com_watabou_utils_ArrayExtender.max(points, (function (out, gate) {
							return function (v) {
								var dir = v.subtract(gate[0].point);
								return (dir.x * out[0].x + dir.y * out[0].y) / dir.get_length();
							};
						})(out, gate));
						var face = outer.face;
						var v1 = gate[0];
						var v2 = model.dcel.vertices.h[farthest.__id__];
						var newEdge = model.dcel.splitFace(face, v1, v2);
						var newFaces = [newEdge.face, newEdge.twin.face];
						var model1 = model.patches;
						var _g3 = [];
						var _g4 = 0;
						while (_g4 < newFaces.length) {
							var f = newFaces[_g4];
							++_g4;
							_g3.push(f.data = new com_watabou_mfcg_model_Patch(f.getPoly(), f));
						}
						com_watabou_utils_ArrayExtender.replace(model1, outer, _g3);
						var _g5 = 0;
						while (_g5 < newFaces.length) {
							var f1 = newFaces[_g5];
							++_g5;
							var e = f1.halfEdge;
							while (true) {
								if (e.twin.data == com_watabou_mfcg_model_PatchEdge.WALL) {
									e.data = com_watabou_mfcg_model_PatchEdge.WALL;
								}
								e = e.next;
								if (!(e != f1.halfEdge)) {
									break;
								}
							}
						}
					}
				}
			}
			var _g6 = 0;
			var _g7 = weights.length;
			while (_g6 < _g7) {
				var i3 = _g6++;
				var d = Math.abs(i3 - index);
				if (d > weights.length / 2) {
					d = weights.length - d;
				}
				weights[i3] *= d <= 1 ? 0 : d - 1;
			}
		}
		if (this.gates.length == 0 && maxGates > 0) {
			throw new openfl_errors_Error("No gates created!");
		}
		if (real) {
			var _g = 0;
			var _g1 = this.gates;
			while (_g < _g1.length) {
				var gate1 = _g1[_g];
				++_g;
				com_watabou_utils_PointExtender.set(gate1.point, com_watabou_mfcg_utils_PolyUtils.lerpVertex(this.shape, gate1.point));
			}
		}
	}
	, buildCastleGate: function (model, reserved) {
		var _g = 0;
		var _g1 = this.edges;
		while (_g < _g1.length) {
			var edge = _g1[_g];
			++_g;
			if (edge.twin.face.data == model.plaza) {
				this.gates = [this.splitSegment(model, edge)];
				return;
			}
		}
		var available = com_watabou_utils_ArrayExtender.difference(this.shape, reserved);
		if (available.length == 0) {
			var _g = [];
			var _g1 = 0;
			var _g2 = this.edges;
			while (_g1 < _g2.length) {
				var v = _g2[_g1];
				++_g1;
				if (v.twin.face.data.withinCity) {
					_g.push(v);
				}
			}
			var candidates = _g;
			var edge = com_watabou_utils_ArrayExtender.min(candidates, function (e) {
				return com_watabou_geom_GeomUtils.lerp(e.origin.point, e.next.origin.point, 0.5).get_length();
			});
			this.gates = [this.splitSegment(model, edge)];
		} else {
			var gate = com_watabou_utils_ArrayExtender.min(available, function (p) {
				return p.get_length();
			});
			com_watabou_utils_PointExtender.set(gate, com_watabou_mfcg_utils_PolyUtils.lerpVertex(this.shape, gate));
			this.gates = [model.dcel.vertices.h[gate.__id__]];
		}
	}
	, splitSegment: function (model, edge) {
		var e = model.splitEdge(edge);
		var _g = [];
		var _g1_startEdge = this.patches[0].face.halfEdge;
		var _g1_curEdge = _g1_startEdge;
		var _g1__hasNext = true;
		while (_g1__hasNext) {
			var result = _g1_curEdge;
			_g1_curEdge = _g1_curEdge.next;
			_g1__hasNext = _g1_curEdge != _g1_startEdge;
			var edge = result;
			_g.push(edge);
		}
		this.edges = _g;
		this.shape = this.patches[0].shape;
		this.length++;
		com_watabou_geom_EdgeChain.assignData(this.edges, com_watabou_mfcg_model_PatchEdge.WALL, false);
		return e.origin;
	}
	, buildTowers: function () {
		this.towers = [];
		if (this.real) {
			var _g = 0;
			var _g1 = this.length;
			while (_g < _g1) {
				var i = _g++;
				var t = this.edges[i].origin;
				if (this.gates.indexOf(t) == -1 && (this.segments[(i + this.length - 1) % this.length] || this.segments[i])) {
					this.towers.push(t);
				}
			}
		}
	}
	, bothSegments: function (i) {
		if (this.segments[i]) {
			return this.segments[(i + this.length - 1) % this.length];
		} else {
			return false;
		}
	}
	, addWatergate: function (v, canal) {
		this.watergates.set(v, canal);
		HxOverrides.remove(this.towers, v);
	}
	, getTowerRadius: function (v) {
		if (this.real) {
			if (this.towers.indexOf(v) != -1) {
				return com_watabou_mfcg_model_CurtainWall.LTOWER_RADIUS;
			} else if (this.gates.indexOf(v) != -1) {
				return 1. + com_watabou_mfcg_model_CurtainWall.TOWER_RADIUS * 2;
			} else {
				return 0.0;
			}
		} else {
			return 0.0;
		}
	}
	, __class__: com_watabou_mfcg_model_CurtainWall
};
var com_watabou_mfcg_model_DistrictType = $hxEnums["com.watabou.mfcg.model.DistrictType"] = {
	__ename__: "com.watabou.mfcg.model.DistrictType", __constructs__: null
	, CENTER: ($_ = function (plaza) { return { _hx_index: 0, plaza: plaza, __enum__: "com.watabou.mfcg.model.DistrictType", toString: $estr }; }, $_._hx_name = "CENTER", $_.__params__ = ["plaza"], $_)
	, CASTLE: ($_ = function (castle) { return { _hx_index: 1, castle: castle, __enum__: "com.watabou.mfcg.model.DistrictType", toString: $estr }; }, $_._hx_name = "CASTLE", $_.__params__ = ["castle"], $_)
	, DOCKS: { _hx_name: "DOCKS", _hx_index: 2, __enum__: "com.watabou.mfcg.model.DistrictType", toString: $estr }
	, BRIDGE: ($_ = function (bridge) { return { _hx_index: 3, bridge: bridge, __enum__: "com.watabou.mfcg.model.DistrictType", toString: $estr }; }, $_._hx_name = "BRIDGE", $_.__params__ = ["bridge"], $_)
	, GATE: ($_ = function (gate) { return { _hx_index: 4, gate: gate, __enum__: "com.watabou.mfcg.model.DistrictType", toString: $estr }; }, $_._hx_name = "GATE", $_.__params__ = ["gate"], $_)
	, BANK: { _hx_name: "BANK", _hx_index: 5, __enum__: "com.watabou.mfcg.model.DistrictType", toString: $estr }
	, PARK: { _hx_name: "PARK", _hx_index: 6, __enum__: "com.watabou.mfcg.model.DistrictType", toString: $estr }
	, SPRAWL: { _hx_name: "SPRAWL", _hx_index: 7, __enum__: "com.watabou.mfcg.model.DistrictType", toString: $estr }
	, REGULAR: { _hx_name: "REGULAR", _hx_index: 8, __enum__: "com.watabou.mfcg.model.DistrictType", toString: $estr }
};
com_watabou_mfcg_model_DistrictType.__constructs__ = [com_watabou_mfcg_model_DistrictType.CENTER, com_watabou_mfcg_model_DistrictType.CASTLE, com_watabou_mfcg_model_DistrictType.DOCKS, com_watabou_mfcg_model_DistrictType.BRIDGE, com_watabou_mfcg_model_DistrictType.GATE, com_watabou_mfcg_model_DistrictType.BANK, com_watabou_mfcg_model_DistrictType.PARK, com_watabou_mfcg_model_DistrictType.SPRAWL, com_watabou_mfcg_model_DistrictType.REGULAR];
var com_watabou_mfcg_model_District = function (patches, type) {
	this.type = type;
	var _g = [];
	var _g1 = 0;
	while (_g1 < patches.length) {
		var p = patches[_g1];
		++_g1;
		_g.push(p.face);
	}
	this.faces = _g;
	var _g = 0;
	while (_g < patches.length) {
		var patch = patches[_g];
		++_g;
		patch.district = this;
	}
	this.createParams();
};
$hxClasses["com.watabou.mfcg.model.District"] = com_watabou_mfcg_model_District;
com_watabou_mfcg_model_District.__name__ = "com.watabou.mfcg.model.District";
com_watabou_mfcg_model_District.check4holes = function (faces) {
	var edges = [];
	var _g = 0;
	while (_g < faces.length) {
		var face = faces[_g];
		++_g;
		var edge = face.halfEdge;
		while (true) {
			if (edge.twin == null || faces.indexOf(edge.twin.face) == -1) {
				edges.push(edge);
			}
			edge = edge.next;
			if (!(edge != face.halfEdge)) {
				break;
			}
		}
	}
	var circuit = 0;
	var start = edges[0];
	var edge = start;
	while (true) {
		++circuit;
		edge = edge.next;
		while (edges.indexOf(edge) == -1) edge = edge.twin.next;
		if (!(edge != start)) {
			break;
		}
	}
	return edges.length > circuit;
};
com_watabou_mfcg_model_District.updateColors = function (districts) {
	var n = districts.length;
	if (n == 1) {
		districts[0].color = com_watabou_mfcg_mapping_Style.colorRoof;
	} else {
		var _g = 0;
		var _g1 = n;
		while (_g < _g1) {
			var i = _g++;
			districts[i].color = com_watabou_mfcg_mapping_Style.getTint(com_watabou_mfcg_mapping_Style.colorRoof, i, n);
		}
	}
};
com_watabou_mfcg_model_District.prototype = {
	createParams: function () {
		this.alleys = { minSq: 15 + 40 * Math.abs(((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 2 - 1), gridChaos: 0.2 + ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 3 * 0.8, sizeChaos: 0.4 + ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 3 * 0.6, shapeFactor: 0.25 + ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 3 * 2.0, inset: 0.6 * (1 - Math.abs(((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 2 - 1)), blockSize: 4 + 10 * (((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 3) };
		this.alleys.minFront = Math.sqrt(this.alleys.minSq);
		this.greenery = Math.pow(((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 3, 2);
		if (this.type == com_watabou_mfcg_model_DistrictType.SPRAWL) {
			this.alleys.gridChaos *= 0.5;
			this.alleys.blockSize *= 2.0;
			this.greenery = (1 + this.greenery) / 2;
		}
	}
	, updateGeometry: function () {
		var tmp;
		if (this.faces.length == 1) {
			var _g = [];
			var _g1_startEdge = this.faces[0].halfEdge;
			var _g1_curEdge = _g1_startEdge;
			var _g1__hasNext = true;
			while (_g1__hasNext) {
				var result = _g1_curEdge;
				_g1_curEdge = _g1_curEdge.next;
				_g1__hasNext = _g1_curEdge != _g1_startEdge;
				var edge = result;
				_g.push(edge);
			}
			tmp = _g;
		} else {
			tmp = com_watabou_geom_DCEL.circumference(null, com_watabou_utils_SetUtils.fromArray(this.faces));
		}
		this.border = tmp;
		var area = new com_watabou_mfcg_annotations_Area(com_watabou_geom_EdgeChain.toPoly(this.border));
		if (com_watabou_system_State.get("districts", "Curved") == "Curved") {
			this.label = area.arcLabel();
			if (this.faces.length == 1) {
				this.label.radius *= 2;
			}
		} else {
			this.label = area.horLabel();
		}
	}
	, createGroups: function () {
		var _g = [];
		var _g1 = 0;
		var _g2 = this.faces;
		while (_g1 < _g2.length) {
			var v = _g2[_g1];
			++_g1;
			if (((v.data.ward) instanceof com_watabou_mfcg_model_wards_Alleys)) {
				_g.push(v);
			}
		}
		var faces = _g;
		var tmp;
		if (com_watabou_mfcg_model_District.useGroups) {
			var _g = [];
			while (faces.length > 0) {
				var group = this.pickPatch(faces);
				var _g1 = [];
				var _g2 = 0;
				while (_g2 < group.length) {
					var f = group[_g2];
					++_g2;
					_g1.push(f.data);
				}
				_g.push(new com_watabou_mfcg_model_wards_WardGroup(_g1));
			}
			tmp = _g;
		} else {
			var _g = [];
			var _g1 = 0;
			while (_g1 < faces.length) {
				var face = faces[_g1];
				++_g1;
				_g.push(new com_watabou_mfcg_model_wards_WardGroup([face.data]));
			}
			tmp = _g;
		}
		this.groups = tmp;
	}
	, pickPatch: function (faces) {
		var group = [com_watabou_utils_ArrayExtender.pick(faces)];
		while (true) {
			var candidates = [];
			var _g = 0;
			while (_g < group.length) {
				var face = group[_g];
				++_g;
				var _g_startEdge = face.halfEdge;
				var _g_curEdge = _g_startEdge;
				var _g__hasNext = true;
				while (_g__hasNext) {
					var result = _g_curEdge;
					_g_curEdge = _g_curEdge.next;
					_g__hasNext = _g_curEdge != _g_startEdge;
					var edge = result;
					if (edge.data == null) {
						var n = edge.twin.face;
						if (faces.indexOf(n) != -1 && group.indexOf(n) == -1) {
							candidates.push(n);
						}
					}
				}
			}
			if (candidates.length == 0) {
				break;
			} else {
				var face1 = com_watabou_utils_ArrayExtender.pick(candidates);
				HxOverrides.remove(faces, face1);
				group.push(face1);
			}
		}
		if (group.length > 1 && com_watabou_mfcg_model_District.check4holes(group)) {
			haxe_Log.trace("Hole in a group, we need to split it", { fileName: "Source/com/watabou/mfcg/model/District.hx", lineNumber: 144, className: "com.watabou.mfcg.model.District", methodName: "pickPatch" });
			com_watabou_utils_ArrayExtender.addAll(faces, group.slice(1));
			group = [group[0]];
		}
		return group;
	}
	, center: function () {
		return this.label.pos;
	}
	, __class__: com_watabou_mfcg_model_District
};
var com_watabou_mfcg_model_DistrictBuilder = function (model) {
	this.model = model;
};
$hxClasses["com.watabou.mfcg.model.DistrictBuilder"] = com_watabou_mfcg_model_DistrictBuilder;
com_watabou_mfcg_model_DistrictBuilder.__name__ = "com.watabou.mfcg.model.DistrictBuilder";
com_watabou_mfcg_model_DistrictBuilder.prototype = {
	build: function () {
		var _gthis = this;
		var _g = [];
		var _g1 = 0;
		var _g2 = this.model.patches;
		while (_g1 < _g2.length) {
			var patch = _g2[_g1];
			++_g1;
			if (patch.withinCity) {
				_g.push(patch);
			}
		}
		this.city = _g;
		this.unassigned = this.city.slice();
		this.districts = [];
		var candidates = [];
		var addPoint = function (vertex, type) {
			candidates.push({ patch: null, vertex: vertex, type: type });
		};
		var addPatch = function (patch, type) {
			candidates.push({ patch: patch, vertex: null, type: type != null ? type : _gthis.getType(patch) });
		};
		if (this.model.citadel != null) {
			addPoint((js_Boot.__cast(this.model.citadel.ward, com_watabou_mfcg_model_wards_Castle)).wall.gates[0], com_watabou_mfcg_model_DistrictType.CASTLE(this.model.citadel));
		}
		if (this.model.plaza != null) {
			addPatch(this.model.plaza, com_watabou_mfcg_model_DistrictType.CENTER(this.model.plaza));
		} else {
			addPoint(this.model.dcel.vertices.h[this.model.center.__id__], com_watabou_mfcg_model_DistrictType.CENTER(null));
		}
		var _g = 0;
		var _g1 = this.unassigned;
		while (_g < _g1.length) {
			var patch = _g1[_g];
			++_g;
			if (((patch.ward) instanceof com_watabou_mfcg_model_wards_Park)) {
				addPatch(patch, com_watabou_mfcg_model_DistrictType.PARK);
			}
		}
		if (this.model.wall != null) {
			var _g = 0;
			var _g1 = this.model.wall.gates;
			while (_g < _g1.length) {
				var gate = _g1[_g];
				++_g;
				addPoint(gate, com_watabou_mfcg_model_DistrictType.GATE(gate));
			}
		}
		var _g = 0;
		var _g1 = this.model.canals;
		while (_g < _g1.length) {
			var canal = _g1[_g];
			++_g;
			var bridge = canal.bridges.keys();
			while (bridge.hasNext()) {
				var bridge1 = bridge.next();
				addPoint(bridge1, com_watabou_mfcg_model_DistrictType.BRIDGE(bridge1));
			}
			var v = com_watabou_utils_ArrayExtender.random(canal.course);
			var patch = com_watabou_utils_ArrayExtender.random(canal.course);
			if (patch.face.data.withinCity) {
				addPatch(patch.face.data, com_watabou_mfcg_model_DistrictType.BANK);
			}
			var patch1 = com_watabou_utils_ArrayExtender.random(canal.course).twin;
			if (patch1.face.data.withinCity) {
				addPatch(patch1.face.data, com_watabou_mfcg_model_DistrictType.BANK);
			}
		}
		var _g = 0;
		var _g1 = this.unassigned;
		while (_g < _g1.length) {
			var patch = _g1[_g];
			++_g;
			if (patch.landing && ((patch.ward) instanceof com_watabou_mfcg_model_wards_Alleys)) {
				addPatch(patch);
				break;
			}
		}
		var _g = 0;
		var _g1 = candidates.length;
		while (_g < _g1) {
			var i = _g++;
			addPatch(com_watabou_utils_ArrayExtender.random(this.city));
		}
		var nCores = Math.sqrt(this.city.length) | 0;
		candidates = com_watabou_utils_ArrayExtender.subset(candidates, nCores);
		while (candidates.length < nCores) addPatch(com_watabou_utils_ArrayExtender.random(this.city));
		var _g = [];
		var _g1 = 0;
		var _g2 = candidates;
		while (_g1 < _g2.length) {
			var v = _g2[_g1];
			++_g1;
			if (v.type == com_watabou_mfcg_model_DistrictType.DOCKS) {
				_g.push(v);
			}
		}
		var docks = _g;
		if (docks.length > 1) {
			com_watabou_utils_ArrayExtender.removeAll(candidates, docks);
			candidates.push(docks[0]);
		}
		var _g = 0;
		while (_g < candidates.length) {
			var c = candidates[_g];
			++_g;
			if (c.patch != null) {
				this.fromPatch(c.patch, c.type);
			} else {
				this.fromVertex(c.vertex, c.type);
			}
		}
		this.growAll();
		com_watabou_mfcg_model_District.updateColors(this.districts);
		var _g = 0;
		var _g1 = this.districts;
		while (_g < _g1.length) {
			var district = _g1[_g];
			++_g;
			district.updateGeometry();
			district.createGroups();
		}
		this.sort(this.districts, this.model.patches[0]);
		new com_watabou_mfcg_linguistics_DistrictNames(this.model, this.districts).generate();
	}
	, fromPatch: function (patch, type) {
		if (patch.district == null) {
			var district = new com_watabou_mfcg_model_District([patch], type);
			this.districts.push(district);
			HxOverrides.remove(this.unassigned, patch);
			return district;
		} else {
			return null;
		}
	}
	, fromVertex: function (v, type, needAll) {
		if (needAll == null) {
			needAll = true;
		}
		var patches = this.model.patchesByVertex(v);
		patches = com_watabou_utils_ArrayExtender.intersect(patches, this.city);
		var filtered = [];
		var _g = 0;
		while (_g < patches.length) {
			var patch = patches[_g];
			++_g;
			if (patch.district == null) {
				filtered.push(patch);
			}
		}
		if (filtered.length == 0 || needAll && filtered.length < patches.length) {
			return null;
		}
		var district = new com_watabou_mfcg_model_District(filtered, type);
		this.districts.push(district);
		com_watabou_utils_ArrayExtender.removeAll(this.unassigned, filtered);
		return district;
	}
	, getType: function (patch) {
		if (((patch.ward) instanceof com_watabou_mfcg_model_wards_Castle)) {
			return com_watabou_mfcg_model_DistrictType.CASTLE(patch);
		} else if (((patch.ward) instanceof com_watabou_mfcg_model_wards_Park)) {
			return com_watabou_mfcg_model_DistrictType.PARK;
		} else if (patch.landing && ((patch.ward) instanceof com_watabou_mfcg_model_wards_Alleys)) {
			return com_watabou_mfcg_model_DistrictType.DOCKS;
		} else if (this.model.inner.indexOf(patch) != -1) {
			return com_watabou_mfcg_model_DistrictType.REGULAR;
		} else {
			return com_watabou_mfcg_model_DistrictType.SPRAWL;
		}
	}
	, growAll: function () {
		var _g = [];
		var _g1 = 0;
		var _g2 = this.districts;
		while (_g1 < _g2.length) {
			var d = _g2[_g1];
			++_g1;
			_g.push(this.getGrower(d));
		}
		var growing = _g;
		while (growing.length > 0) {
			var iteration = growing.slice();
			var _g = 0;
			while (_g < iteration.length) {
				var grower = iteration[_g];
				++_g;
				var grown = grower.grow(this.unassigned);
				if (!grown) {
					HxOverrides.remove(growing, grower);
				}
				if (this.unassigned.length == 0) {
					return;
				}
			}
		}
		while (this.unassigned.length > 0) {
			var seed = com_watabou_utils_ArrayExtender.random(this.unassigned);
			var district = this.fromPatch(seed, this.getType(seed));
			var grower = this.getGrower(district);
			while (grower.grow(this.unassigned)) {
			}
		}
	}
	, getGrower: function (d) {
		switch (d.type._hx_index) {
			case 2:
				return new com_watabou_mfcg_model_DocksGrower(d);
			case 6:
				return new com_watabou_mfcg_model_ParkGrower(d);
			default:
				return new com_watabou_mfcg_model_Grower(d);
		}
	}
	, sort: function (districts, p0) {
		var center = com_watabou_geom_polygons_PolyCore.centroid(p0.shape);
		districts.sort(function (d1, d2) {
			var p1 = openfl_geom_Point.distance(center, d1.label.pos);
			var p2 = openfl_geom_Point.distance(center, d2.label.pos);
			var value = p1 - p2;
			if (value == 0) {
				return 0;
			} else if (value < 0) {
				return -1;
			} else {
				return 1;
			}
		});
		var _g = 0;
		while (_g < districts.length) {
			var d = districts[_g];
			++_g;
			if (d.faces.indexOf(p0.face) != -1) {
				HxOverrides.remove(districts, d);
				districts.unshift(d);
				break;
			}
		}
		var last = districts.shift();
		var sorted = [last];
		while (districts.length > 0) {
			var next = com_watabou_utils_ArrayExtender.min(districts, function (d) {
				return openfl_geom_Point.distance(d.label.pos, last.label.pos);
			});
			if (openfl_geom_Point.distance(last.label.pos, next.label.pos) > openfl_geom_Point.distance(center, districts[0].label.pos)) {
				last = districts[0];
			} else {
				last = next;
			}
			HxOverrides.remove(districts, last);
			sorted.push(last);
		}
		var _g = 0;
		while (_g < sorted.length) {
			var e = sorted[_g];
			++_g;
			districts.push(e);
		}
	}
	, __class__: com_watabou_mfcg_model_DistrictBuilder
};
var com_watabou_mfcg_model_Grower = function (district) {
	this.district = district;
	var _g = district.type;
	var tmp;
	switch (_g._hx_index) {
		case 1:
			var castle = _g.castle;
			tmp = 0.1;
			break;
		case 3:
			var bridge = _g.bridge;
			tmp = 0.1;
			break;
		case 4:
			var gate = _g.gate;
			tmp = 0.1;
			break;
		case 5:
			tmp = 0.5;
			break;
		default:
			tmp = 1.0;
	}
	this.rate = tmp;
};
$hxClasses["com.watabou.mfcg.model.Grower"] = com_watabou_mfcg_model_Grower;
com_watabou_mfcg_model_Grower.__name__ = "com.watabou.mfcg.model.Grower";
com_watabou_mfcg_model_Grower.prototype = {
	grow: function (unassigned) {
		if (this.rate == 0) {
			return false;
		} else {
			var chance = 1 - this.rate;
			if (chance == null) {
				chance = 0.5;
			}
			if ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance) {
				return true;
			}
		}
		var candidates = [];
		var _g = 0;
		var _g1 = this.district.faces;
		while (_g < _g1.length) {
			var face = _g1[_g];
			++_g;
			var edge = face.halfEdge;
			while (true) {
				var n = edge.twin != null ? edge.twin.face.data : null;
				if (unassigned.indexOf(n) != -1) {
					var chance = this.validatePatch(face.data, n) * this.validateEdge(edge.data);
					var chance1 = chance;
					if (chance1 == null) {
						chance1 = 0.5;
					}
					if ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance1) {
						com_watabou_utils_ArrayExtender.add(candidates, n);
					}
				}
				edge = edge.next;
				if (!(edge != face.halfEdge)) {
					break;
				}
			}
		}
		var _g = 0;
		while (_g < candidates.length) {
			var p = candidates[_g];
			++_g;
			if (p.district == null) {
				p.district = this.district;
				HxOverrides.remove(unassigned, p);
				this.district.faces.push(p.face);
			}
		}
		return candidates.length > 0;
	}
	, validatePatch: function (p, n) {
		if (p.landing == n.landing) {
			return 1;
		} else {
			return 0;
		}
	}
	, validateEdge: function (e) {
		if (e != null) {
			switch (e._hx_index) {
				case 2:
					return 0.9;
				case 3:
					return 0;
				case 4:
					var canal = e.c;
					return 0;
				default:
					return 1;
			}
		} else {
			return 1;
		}
	}
	, __class__: com_watabou_mfcg_model_Grower
};
var com_watabou_mfcg_model_DocksGrower = function (district) {
	com_watabou_mfcg_model_Grower.call(this, district);
};
$hxClasses["com.watabou.mfcg.model.DocksGrower"] = com_watabou_mfcg_model_DocksGrower;
com_watabou_mfcg_model_DocksGrower.__name__ = "com.watabou.mfcg.model.DocksGrower";
com_watabou_mfcg_model_DocksGrower.__super__ = com_watabou_mfcg_model_Grower;
com_watabou_mfcg_model_DocksGrower.prototype = $extend(com_watabou_mfcg_model_Grower.prototype, {
	validatePatch: function (p, n) {
		if (n.landing && ((n.ward) instanceof com_watabou_mfcg_model_wards_Alleys)) {
			return 1;
		} else {
			return 0;
		}
	}
	, __class__: com_watabou_mfcg_model_DocksGrower
});
var com_watabou_mfcg_model_ParkGrower = function (district) {
	com_watabou_mfcg_model_Grower.call(this, district);
};
$hxClasses["com.watabou.mfcg.model.ParkGrower"] = com_watabou_mfcg_model_ParkGrower;
com_watabou_mfcg_model_ParkGrower.__name__ = "com.watabou.mfcg.model.ParkGrower";
com_watabou_mfcg_model_ParkGrower.__super__ = com_watabou_mfcg_model_Grower;
com_watabou_mfcg_model_ParkGrower.prototype = $extend(com_watabou_mfcg_model_Grower.prototype, {
	validatePatch: function (p, n) {
		if (((n.ward) instanceof com_watabou_mfcg_model_wards_Park)) {
			return 1;
		} else {
			return 0;
		}
	}
	, __class__: com_watabou_mfcg_model_ParkGrower
});
var com_watabou_mfcg_model_Landmark = function (model, pos, name) {
	if (name == null) {
		name = "Landmark";
	}
	this.model = model;
	this.pos = pos;
	this.name = name;
	this.assign();
};
$hxClasses["com.watabou.mfcg.model.Landmark"] = com_watabou_mfcg_model_Landmark;
com_watabou_mfcg_model_Landmark.__name__ = "com.watabou.mfcg.model.Landmark";
com_watabou_mfcg_model_Landmark.prototype = {
	assign: function () {
		var _g = 0;
		var _g1 = this.model.patches;
		while (_g < _g1.length) {
			var patch = _g1[_g];
			++_g;
			if (this.assignPoly(patch.shape)) {
				return;
			}
		}
	}
	, assignPoly: function (poly) {
		if (com_watabou_geom_polygons_PolyBounds.rect(poly).containsPoint(this.pos)) {
			var n = poly.length;
			this.p0 = poly[0];
			var _g = 2;
			var _g1 = n;
			while (_g < _g1) {
				var i = _g++;
				this.p1 = poly[i - 1];
				this.p2 = poly[i];
				var b = com_watabou_geom_GeomUtils.barycentric(this.p0, this.p1, this.p2, this.pos);
				if (b.x >= 0 && b.y >= 0 && b.z >= 0) {
					this.i0 = b.x;
					this.i1 = b.y;
					this.i2 = b.z;
					return true;
				}
			}
		}
		return false;
	}
	, update: function () {
		var p = this.p0;
		var f = this.i0;
		var tmp = new openfl_geom_Point(p.x * f, p.y * f);
		var p = this.p1;
		var f = this.i1;
		var tmp1 = tmp.add(new openfl_geom_Point(p.x * f, p.y * f));
		var p = this.p2;
		var f = this.i2;
		this.pos = tmp1.add(new openfl_geom_Point(p.x * f, p.y * f));
	}
	, __class__: com_watabou_mfcg_model_Landmark
};
var msignal_Signal = function (valueClasses) {
	if (valueClasses == null) {
		valueClasses = [];
	}
	this.valueClasses = valueClasses;
	this.slots = msignal_SlotList.NIL;
	this.priorityBased = false;
};
$hxClasses["msignal.Signal"] = msignal_Signal;
msignal_Signal.__name__ = "msignal.Signal";
msignal_Signal.prototype = {
	add: function (listener) {
		return this.registerListener(listener);
	}
	, addOnce: function (listener) {
		return this.registerListener(listener, true);
	}
	, addWithPriority: function (listener, priority) {
		if (priority == null) {
			priority = 0;
		}
		return this.registerListener(listener, false, priority);
	}
	, addOnceWithPriority: function (listener, priority) {
		if (priority == null) {
			priority = 0;
		}
		return this.registerListener(listener, true, priority);
	}
	, remove: function (listener) {
		var slot = this.slots.find(listener);
		if (slot == null) {
			return null;
		}
		this.slots = this.slots.filterNot(listener);
		return slot;
	}
	, removeAll: function () {
		this.slots = msignal_SlotList.NIL;
	}
	, registerListener: function (listener, once, priority) {
		if (priority == null) {
			priority = 0;
		}
		if (once == null) {
			once = false;
		}
		if (this.registrationPossible(listener, once)) {
			var newSlot = this.createSlot(listener, once, priority);
			if (!this.priorityBased && priority != 0) {
				this.priorityBased = true;
			}
			if (!this.priorityBased && priority == 0) {
				this.slots = this.slots.prepend(newSlot);
			} else {
				this.slots = this.slots.insertWithPriority(newSlot);
			}
			return newSlot;
		}
		return this.slots.find(listener);
	}
	, registrationPossible: function (listener, once) {
		if (!this.slots.nonEmpty) {
			return true;
		}
		var existingSlot = this.slots.find(listener);
		if (existingSlot == null) {
			return true;
		}
		return false;
	}
	, createSlot: function (listener, once, priority) {
		if (priority == null) {
			priority = 0;
		}
		if (once == null) {
			once = false;
		}
		return null;
	}
	, get_numListeners: function () {
		return this.slots.get_length();
	}
	, __class__: msignal_Signal
	, __properties__: { get_numListeners: "get_numListeners" }
};
var msignal_Signal0 = function () {
	msignal_Signal.call(this);
};
$hxClasses["msignal.Signal0"] = msignal_Signal0;
msignal_Signal0.__name__ = "msignal.Signal0";
msignal_Signal0.__super__ = msignal_Signal;
msignal_Signal0.prototype = $extend(msignal_Signal.prototype, {
	dispatch: function () {
		var slotsToProcess = this.slots;
		while (slotsToProcess.nonEmpty) {
			slotsToProcess.head.execute();
			slotsToProcess = slotsToProcess.tail;
		}
	}
	, createSlot: function (listener, once, priority) {
		if (priority == null) {
			priority = 0;
		}
		if (once == null) {
			once = false;
		}
		return new msignal_Slot0(this, listener, once, priority);
	}
	, __class__: msignal_Signal0
});
var msignal_SlotList = function (head, tail) {
	this.nonEmpty = false;
	if (head == null && tail == null) {
		this.nonEmpty = false;
	} else if (head != null) {
		this.head = head;
		this.tail = tail == null ? msignal_SlotList.NIL : tail;
		this.nonEmpty = true;
	}
};
$hxClasses["msignal.SlotList"] = msignal_SlotList;
msignal_SlotList.__name__ = "msignal.SlotList";
msignal_SlotList.prototype = {
	get_length: function () {
		if (!this.nonEmpty) {
			return 0;
		}
		if (this.tail == msignal_SlotList.NIL) {
			return 1;
		}
		var result = 0;
		var p = this;
		while (p.nonEmpty) {
			++result;
			p = p.tail;
		}
		return result;
	}
	, prepend: function (slot) {
		return new msignal_SlotList(slot, this);
	}
	, append: function (slot) {
		if (slot == null) {
			return this;
		}
		if (!this.nonEmpty) {
			return new msignal_SlotList(slot);
		}
		if (this.tail == msignal_SlotList.NIL) {
			return new msignal_SlotList(slot).prepend(this.head);
		}
		var wholeClone = new msignal_SlotList(this.head);
		var subClone = wholeClone;
		var current = this.tail;
		while (current.nonEmpty) {
			subClone = subClone.tail = new msignal_SlotList(current.head);
			current = current.tail;
		}
		subClone.tail = new msignal_SlotList(slot);
		return wholeClone;
	}
	, insertWithPriority: function (slot) {
		if (!this.nonEmpty) {
			return new msignal_SlotList(slot);
		}
		var priority = slot.priority;
		if (priority >= this.head.priority) {
			return this.prepend(slot);
		}
		var wholeClone = new msignal_SlotList(this.head);
		var subClone = wholeClone;
		var current = this.tail;
		while (current.nonEmpty) {
			if (priority > current.head.priority) {
				subClone.tail = current.prepend(slot);
				return wholeClone;
			}
			subClone = subClone.tail = new msignal_SlotList(current.head);
			current = current.tail;
		}
		subClone.tail = new msignal_SlotList(slot);
		return wholeClone;
	}
	, filterNot: function (listener) {
		if (!this.nonEmpty || listener == null) {
			return this;
		}
		if (Reflect.compareMethods(this.head.listener, listener)) {
			return this.tail;
		}
		var wholeClone = new msignal_SlotList(this.head);
		var subClone = wholeClone;
		var current = this.tail;
		while (current.nonEmpty) {
			if (Reflect.compareMethods(current.head.listener, listener)) {
				subClone.tail = current.tail;
				return wholeClone;
			}
			subClone = subClone.tail = new msignal_SlotList(current.head);
			current = current.tail;
		}
		return this;
	}
	, contains: function (listener) {
		if (!this.nonEmpty) {
			return false;
		}
		var p = this;
		while (p.nonEmpty) {
			if (Reflect.compareMethods(p.head.listener, listener)) {
				return true;
			}
			p = p.tail;
		}
		return false;
	}
	, find: function (listener) {
		if (!this.nonEmpty) {
			return null;
		}
		var p = this;
		while (p.nonEmpty) {
			if (Reflect.compareMethods(p.head.listener, listener)) {
				return p.head;
			}
			p = p.tail;
		}
		return null;
	}
	, __class__: msignal_SlotList
	, __properties__: { get_length: "get_length" }
};
var msignal_Signal1 = function (type) {
	msignal_Signal.call(this, [type]);
};
$hxClasses["msignal.Signal1"] = msignal_Signal1;
msignal_Signal1.__name__ = "msignal.Signal1";
msignal_Signal1.__super__ = msignal_Signal;
msignal_Signal1.prototype = $extend(msignal_Signal.prototype, {
	dispatch: function (value) {
		var slotsToProcess = this.slots;
		while (slotsToProcess.nonEmpty) {
			slotsToProcess.head.execute(value);
			slotsToProcess = slotsToProcess.tail;
		}
	}
	, createSlot: function (listener, once, priority) {
		if (priority == null) {
			priority = 0;
		}
		if (once == null) {
			once = false;
		}
		return new msignal_Slot1(this, listener, once, priority);
	}
	, __class__: msignal_Signal1
});
var com_watabou_mfcg_model_ModelDispatcher = function () { };
$hxClasses["com.watabou.mfcg.model.ModelDispatcher"] = com_watabou_mfcg_model_ModelDispatcher;
com_watabou_mfcg_model_ModelDispatcher.__name__ = "com.watabou.mfcg.model.ModelDispatcher";
var com_watabou_mfcg_model_Patch = function (vertices, face) {
	this.seed = -1;
	this.district = null;
	this.face = face;
	this.shape = vertices;
	this.withinCity = false;
	this.withinWalls = false;
	this.waterbody = false;
	this.seed = com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0;
};
$hxClasses["com.watabou.mfcg.model.Patch"] = com_watabou_mfcg_model_Patch;
com_watabou_mfcg_model_Patch.__name__ = "com.watabou.mfcg.model.Patch";
com_watabou_mfcg_model_Patch.prototype = {
	bordersPatch: function (patch) {
		var e = this.face.halfEdge;
		while (true) {
			if (e.twin.face.data == patch) {
				return true;
			}
			e = e.next;
			if (!(e != this.face.halfEdge)) {
				break;
			}
		}
		return false;
	}
	, bordersInside: function (region) {
		var _g = 0;
		while (_g < region.length) {
			var e = region[_g];
			++_g;
			if (e.face == this.face) {
				return true;
			}
		}
		return false;
	}
	, bordersOutside: function (region) {
		var _g = 0;
		while (_g < region.length) {
			var e = region[_g];
			++_g;
			if (e.twin != null && e.twin.face == this.face) {
				return true;
			}
		}
		return false;
	}
	, borders: function (region) {
		var _g = 0;
		while (_g < region.length) {
			var e = region[_g];
			++_g;
			if (e.face == this.face || e.twin.face == this.face) {
				return true;
			}
		}
		return false;
	}
	, isRerollable: function () {
		if (this.view != null) {
			return this.view.parent != null;
		} else {
			return false;
		}
	}
	, reroll: function () {
		if (this.view != null && this.view.parent != null) {
			this.seed = com_watabou_utils_Random.seed;
			this.ward.createGeometry();
			this.view.draw();
			com_watabou_mfcg_model_ModelDispatcher.geometryChanged.dispatch(this);
		}
	}
	, __class__: com_watabou_mfcg_model_Patch
};
var com_watabou_mfcg_model_PatchEdge = $hxEnums["com.watabou.mfcg.model.PatchEdge"] = {
	__ename__: "com.watabou.mfcg.model.PatchEdge", __constructs__: null
	, HORIZON: { _hx_name: "HORIZON", _hx_index: 0, __enum__: "com.watabou.mfcg.model.PatchEdge", toString: $estr }
	, COAST: { _hx_name: "COAST", _hx_index: 1, __enum__: "com.watabou.mfcg.model.PatchEdge", toString: $estr }
	, ROAD: { _hx_name: "ROAD", _hx_index: 2, __enum__: "com.watabou.mfcg.model.PatchEdge", toString: $estr }
	, WALL: { _hx_name: "WALL", _hx_index: 3, __enum__: "com.watabou.mfcg.model.PatchEdge", toString: $estr }
	, CANAL: ($_ = function (c) { return { _hx_index: 4, c: c, __enum__: "com.watabou.mfcg.model.PatchEdge", toString: $estr }; }, $_._hx_name = "CANAL", $_.__params__ = ["c"], $_)
};
com_watabou_mfcg_model_PatchEdge.__constructs__ = [com_watabou_mfcg_model_PatchEdge.HORIZON, com_watabou_mfcg_model_PatchEdge.COAST, com_watabou_mfcg_model_PatchEdge.ROAD, com_watabou_mfcg_model_PatchEdge.WALL, com_watabou_mfcg_model_PatchEdge.CANAL];
var com_watabou_mfcg_model_Topology = function (patches) {
	this.graph = new com_watabou_geom_Graph();
	this.pt2node = new haxe_ds_ObjectMap();
	var _g = 0;
	while (_g < patches.length) {
		var p = patches[_g];
		++_g;
		var e = p.face.halfEdge;
		while (true) {
			if (e.data == null) {
				this.getNode(e.origin).link(this.getNode(e.next.origin), openfl_geom_Point.distance(e.origin.point, e.next.origin.point), false);
			}
			e = e.next;
			if (!(e != p.face.halfEdge)) {
				break;
			}
		}
	}
};
$hxClasses["com.watabou.mfcg.model.Topology"] = com_watabou_mfcg_model_Topology;
com_watabou_mfcg_model_Topology.__name__ = "com.watabou.mfcg.model.Topology";
com_watabou_mfcg_model_Topology.prototype = {
	exists: function (v) {
		return this.pt2node.h.__keys__[v.__id__] != null;
	}
	, getNode: function (v) {
		if (this.pt2node.h.__keys__[v.__id__] != null) {
			return this.pt2node.h[v.__id__];
		} else {
			var this1 = this.pt2node;
			var v1 = this.graph.add(new com_watabou_geom_Node(v));
			this1.set(v, v1);
			return v1;
		}
	}
	, buildPath: function (from, to) {
		var start = this.pt2node.h[from.__id__];
		var end = this.pt2node.h[to.__id__];
		if (start == null || end == null) {
			return null;
		}
		var path = this.graph.aStar(this.pt2node.h[from.__id__], this.pt2node.h[to.__id__]);
		if (path == null) {
			return null;
		} else {
			var _g = [];
			var _g1 = 0;
			while (_g1 < path.length) {
				var n = path[_g1];
				++_g1;
				_g.push(n.data);
			}
			return _g;
		}
	}
	, excludePoints: function (points) {
		var _g = 0;
		while (_g < points.length) {
			var p = points[_g];
			++_g;
			var node = this.pt2node.h[p.__id__];
			if (node != null) {
				node.unlinkAll();
			}
		}
	}
	, excludePolygon: function (poly) {
		var _g = 0;
		while (_g < poly.length) {
			var e = poly[_g];
			++_g;
			var start = this.pt2node.h[e.origin.__id__];
			var end = this.pt2node.h[e.next.origin.__id__];
			if (start != null && end != null) {
				start.unlink(end);
			}
		}
	}
	, __class__: com_watabou_mfcg_model_Topology
};
var com_watabou_utils_Random = function () { };
$hxClasses["com.watabou.utils.Random"] = com_watabou_utils_Random;
com_watabou_utils_Random.__name__ = "com.watabou.utils.Random";
com_watabou_utils_Random.reset = function (seed) {
	if (seed == null) {
		seed = -1;
	}
	com_watabou_utils_Random.seed = seed != -1 ? seed : new Date().getTime() % 2147483647 | 0;
};
com_watabou_utils_Random.save = function () {
	return com_watabou_utils_Random.saved = com_watabou_utils_Random.seed;
};
com_watabou_utils_Random.restore = function (value) {
	if (value == null) {
		value = -1;
	}
	if (value != -1) {
		com_watabou_utils_Random.seed = value;
	} else if (com_watabou_utils_Random.saved != -1) {
		com_watabou_utils_Random.seed = com_watabou_utils_Random.saved;
		com_watabou_utils_Random.saved = -1;
	}
};
com_watabou_utils_Random.getSeed = function () {
	return com_watabou_utils_Random.seed;
};
com_watabou_utils_Random.preserve = function (f) {
	var seed = com_watabou_utils_Random.save();
	var result = f();
	com_watabou_utils_Random.restore(seed);
	return result;
};
com_watabou_utils_Random.time = function () {
	return new Date().getTime() % 2147483647 | 0;
};
com_watabou_utils_Random.next = function () {
	return com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0;
};
com_watabou_utils_Random.float = function () {
	return (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647;
};
com_watabou_utils_Random.float2 = function () {
	var f = (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647;
	return f * f * f;
};
com_watabou_utils_Random.normal = function () {
	return ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 3;
};
com_watabou_utils_Random.normal2 = function () {
	return ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 3 * 2 - 1;
};
com_watabou_utils_Random.small = function () {
	return Math.abs(((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 2 - 1);
};
com_watabou_utils_Random.int = function (min, max) {
	return Math.floor(min + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 * (max - min));
};
com_watabou_utils_Random.int0 = function (max) {
	return Math.floor((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 * max);
};
com_watabou_utils_Random.d = function (max) {
	return 1 + Math.floor((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 * max);
};
com_watabou_utils_Random.roll = function (n) {
	return Math.floor((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 * n) + 1;
};
com_watabou_utils_Random.frac = function (f) {
	var chance = f - (f | 0);
	if (chance == null) {
		chance = 0.5;
	}
	return (f | 0) + ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance ? 1 : 0);
};
com_watabou_utils_Random.bool = function (chance) {
	if (chance == null) {
		chance = 0.5;
	}
	return (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance;
};
com_watabou_utils_Random.fuzzy = function (f) {
	if (f == null) {
		f = 1.0;
	}
	if (f == 0) {
		return 0.5;
	} else {
		return (1 - f) / 2 + f * (((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 3);
	}
};
var com_watabou_mfcg_model_Tree = function (c) {
	com_watabou_geom_Circle.call(this, c, 2.0 * Math.pow(1.5, ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 3 * 2 - 1));
};
$hxClasses["com.watabou.mfcg.model.Tree"] = com_watabou_mfcg_model_Tree;
com_watabou_mfcg_model_Tree.__name__ = "com.watabou.mfcg.model.Tree";
com_watabou_mfcg_model_Tree.fillArea = function (area, density) {
	if (density == null) {
		density = 1.0;
	}
	var points = com_watabou_mfcg_model_Tree.pattern.fill(new com_watabou_geom_FillablePoly(area));
	var _g = [];
	var _g1 = 0;
	while (_g1 < points.length) {
		var p = points[_g1];
		++_g1;
		var chance = density;
		if (chance == null) {
			chance = 0.5;
		}
		if ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance) {
			_g.push(new com_watabou_mfcg_model_Tree(p));
		}
	}
	return _g;
};
com_watabou_mfcg_model_Tree.__super__ = com_watabou_geom_Circle;
com_watabou_mfcg_model_Tree.prototype = $extend(com_watabou_geom_Circle.prototype, {
	__class__: com_watabou_mfcg_model_Tree
});
var com_watabou_mfcg_model_blocks_BisectionBlock = function () { };
$hxClasses["com.watabou.mfcg.model.blocks.BisectionBlock"] = com_watabou_mfcg_model_blocks_BisectionBlock;
com_watabou_mfcg_model_blocks_BisectionBlock.__name__ = "com.watabou.mfcg.model.blocks.BisectionBlock";
com_watabou_mfcg_model_blocks_BisectionBlock.createLots = function (block, params, filterLots) {
	var chaos = params.sizeChaos;
	var cut = null;
	cut = function (p) {
		var obb;
		if (block.cacheOBB.h.__keys__[p.__id__] != null) {
			obb = block.cacheOBB.h[p.__id__];
		} else {
			var this1 = block.cacheOBB;
			var v = com_watabou_geom_polygons_PolyBounds.obb(p);
			this1.set(p, v);
			obb = v;
		}
		var o = obb[0];
		var a = obb[1].subtract(o);
		var al = a.get_length();
		var b = obb[3].subtract(o);
		var bl = b.get_length();
		if (al < bl) {
			var t = a;
			a = b;
			b = t;
			al = bl;
		}
		var m = 1.2 / al;
		var r = m > 0.5 ? 0.5 : m + (1 - 2 * m) * ((((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 3 - 0.5) * chaos + 0.5);
		var p1 = new openfl_geom_Point(o.x + a.x * r, o.y + a.y * r);
		var p2 = p1.add(b);
		var slices = com_watabou_geom_polygons_PolyCut.cut(p, p1, p2, 0, 0.2);
		var lots = [];
		var _g = 0;
		while (_g < slices.length) {
			var slice = slices[_g];
			++_g;
			var cut1;
			if (block.cacheArea.h.__keys__[slice.__id__] != null) {
				cut1 = block.cacheArea.h[slice.__id__];
			} else {
				var this1 = block.cacheArea;
				var v = com_watabou_geom_polygons_PolyCore.area(slice);
				this1.set(slice, v);
				cut1 = v;
			}
			if (cut1 < params.minSq * Math.pow(2, params.sizeChaos * (2 * ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) - 1))) {
				if (slice.length >= 3) {
					lots.push(slice);
				}
			} else {
				var b = cut(slice);
				var _g1 = 0;
				while (_g1 < b.length) {
					var e = b[_g1];
					++_g1;
					lots.push(e);
				}
			}
		}
		return lots;
	};
	return block.filterInner(cut(block.shape), filterLots);
};
var com_watabou_mfcg_model_blocks_Block = function (group, shape, single) {
	if (single == null) {
		single = false;
	}
	this.cacheOBB = new haxe_ds_ObjectMap();
	this.cacheArea = new haxe_ds_ObjectMap();
	this.group = group;
	this.shape = shape;
	var params = group.district.alleys;
	if (single) {
		this.lots = [shape];
	} else if (com_watabou_mfcg_Main.preview) {
		this.lots = com_watabou_mfcg_model_blocks_BisectionBlock.createLots(this, params, true);
	} else {
		var filterLots = com_watabou_system_State.get("no_triangles", false);
		switch (com_watabou_system_State.get("lots_method", "Twisted")) {
			case "Bisection":
				this.lots = com_watabou_mfcg_model_blocks_BisectionBlock.createLots(this, params, filterLots);
				break;
			case "Twisted":
				this.lots = com_watabou_mfcg_model_blocks_TwistedBlock.createLots(this, params, filterLots);
				break;
			case "Voronoi":
				this.lots = com_watabou_mfcg_model_blocks_VoronoiBlock.createLots(this, params, filterLots);
				break;
		}
	}
	if (!com_watabou_mfcg_Main.preview && com_watabou_system_State.get("processing") == "Offset") {
		this.indentFronts(this.lots);
	}
};
$hxClasses["com.watabou.mfcg.model.blocks.Block"] = com_watabou_mfcg_model_blocks_Block;
com_watabou_mfcg_model_blocks_Block.__name__ = "com.watabou.mfcg.model.blocks.Block";
com_watabou_mfcg_model_blocks_Block.randomLotArea = function (params) {
	return params.minSq * Math.pow(2, params.sizeChaos * (2 * ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) - 1));
};
com_watabou_mfcg_model_blocks_Block.randomBlockArea = function (params) {
	return params.minSq * Math.pow(2, params.sizeChaos * (2 * ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) - 1)) * params.blockSize;
};
com_watabou_mfcg_model_blocks_Block.minBlockArea = function (params) {
	return params.minSq * params.blockSize;
};
com_watabou_mfcg_model_blocks_Block.prototype = {
	createRects: function () {
		this.rects = [];
		var inset = this.group.district.alleys.inset;
		var blockLen = this.shape.length;
		var _g = 0;
		var _g1 = this.lots;
		while (_g < _g1.length) {
			var lot = _g1[_g];
			++_g;
			var front = -1;
			var asIs = false;
			var lotLen = lot.length;
			if (this.isRectangle(lot)) {
				asIs = true;
			} else {
				var _g2 = 0;
				var _g3 = lotLen;
				while (_g2 < _g3) {
					var i = _g2++;
					var e0 = lot[i];
					var e1 = lot[(i + 1) % lotLen];
					var alley = -1;
					var _g4 = 0;
					var _g5 = blockLen;
					while (_g4 < _g5) {
						var j = _g4++;
						var a0 = this.shape[j];
						var a1 = this.shape[(j + 1) % blockLen];
						if (com_watabou_geom_GeomUtils.converge(e0, e1, a0, a1)) {
							alley = i;
							break;
						}
					}
					if (alley != -1) {
						if (front != -1) {
							asIs = true;
							break;
						} else {
							front = alley;
						}
					}
				}
			}
			var resultShape;
			if (asIs) {
				resultShape = lot;
			} else {
				var lotArea;
				if (this.cacheArea.h.__keys__[lot.__id__] != null) {
					lotArea = this.cacheArea.h[lot.__id__];
				} else {
					var this1 = this.cacheArea;
					var v = com_watabou_geom_polygons_PolyCore.area(lot);
					this1.set(lot, v);
					lotArea = v;
				}
				var rect = front != -1 ? com_watabou_geom_polygons_PolyBounds.lir(lot, front) : com_watabou_geom_polygons_PolyBounds.lira(lot);
				var minSide = Math.max(1.2, Math.sqrt(lotArea) / 2);
				if (openfl_geom_Point.distance(rect[0], rect[1 % rect.length]) >= minSide && openfl_geom_Point.distance(rect[1], rect[2 % rect.length]) >= minSide) {
					resultShape = rect;
				} else {
					resultShape = lot;
				}
			}
			if (com_watabou_system_State.get("processing") == "Shrink") {
				var i1 = inset * (1 - Math.abs(((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 2 - 1));
				if (i1 > 0.3) {
					var l = resultShape.length;
					var _g6 = [];
					var _g7 = 0;
					var _g8 = l;
					while (_g7 < _g8) {
						var j1 = _g7++;
						var e01 = resultShape[j1];
						var e11 = resultShape[(j1 + 1) % l];
						var alley1 = false;
						var _g9 = 0;
						var _g10 = blockLen;
						while (_g9 < _g10) {
							var j2 = _g9++;
							var a01 = this.shape[j2];
							var a11 = this.shape[(j2 + 1) % blockLen];
							if (com_watabou_geom_GeomUtils.converge(e01, e11, a01, a11)) {
								alley1 = true;
								break;
							}
						}
						_g6.push(alley1 ? 0.0 : i1);
					}
					var d = _g6;
					resultShape = com_watabou_geom_polygons_PolyCut.shrink(resultShape, d);
				}
			}
			this.rects.push(resultShape);
		}
	}
	, isRectangle: function (poly) {
		if (poly.length != 4) {
			return false;
		}
		var a1 = com_watabou_geom_polygons_PolyCore.area(poly);
		var a2 = com_watabou_geom_polygons_PolyCore.rectArea(com_watabou_geom_polygons_PolyBounds.obb(poly));
		return a1 / a2 > 0.75;
	}
	, createBuildings: function () {
		var _gthis = this;
		if (this.rects == null) {
			this.createRects();
		}
		var alleys = this.group.district.alleys;
		var minBlockSq = alleys.minSq / 4 * alleys.shapeFactor;
		this.buildings = [];
		var _g = 0;
		var _g1 = this.rects;
		while (_g < _g1.length) {
			var rect = [_g1[_g]];
			++_g;
			var addBuilding = (function (rect) {
				return function (quad) {
					var b = com_watabou_mfcg_model_Building.create(quad, minBlockSq, true, null, 0.6);
					_gthis.buildings.push(b != null ? b : rect[0]);
				};
			})(rect);
			if (rect[0].length > 4) {
				var newRect = rect[0].slice();
				while (true) {
					com_watabou_geom_polygons_PolyCore.simplifyClosed(newRect);
					if (!(newRect.length > 4)) {
						break;
					}
				}
				addBuilding(newRect);
			} else if (rect[0].length == 4) {
				addBuilding(rect[0]);
			} else {
				this.buildings.push(rect[0]);
			}
		}
	}
	, filterInner: function (lots, noTriangles) {
		if (noTriangles == null) {
			noTriangles = false;
		}
		var _gthis = this;
		var _g = [];
		var _g1 = 0;
		var _g2 = lots;
		while (_g1 < _g2.length) {
			var v = _g2[_g1];
			++_g1;
			if ((function (part) {
				var len = part.length;
				var _g = 0;
				var _g1 = len;
				while (_g < _g1) {
					var i = _g++;
					var v0 = part[i];
					var v1 = part[(i + 1) % len];
					var _g2 = 0;
					var _g3 = _gthis.shape.length;
					while (_g2 < _g3) {
						var j = _g2++;
						var u0 = _gthis.shape[j];
						var u1 = _gthis.shape[(j + 1) % _gthis.shape.length];
						if (com_watabou_geom_GeomUtils.converge(v0, v1, u0, u1)) {
							return false;
						}
					}
				}
				return true;
			})(v)) {
				_g.push(v);
			}
		}
		this.courtyard = _g;
		return com_watabou_utils_ArrayExtender.difference(lots, this.courtyard);
	}
	, indentFronts: function (plans) {
		var max = 1.2;
		var _g = 0;
		var _g1 = plans.length;
		while (_g < _g1) {
			var i = _g++;
			var b = plans[i];
			var side;
			if (this.cacheArea.h.__keys__[b.__id__] != null) {
				side = this.cacheArea.h[b.__id__];
			} else {
				var this1 = this.cacheArea;
				var v = com_watabou_geom_polygons_PolyCore.area(b);
				this1.set(b, v);
				side = v;
			}
			var side1 = Math.sqrt(side);
			var dist = Math.min(side1 / 3, max) * ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647);
			if (dist < 0.5) {
				continue;
			}
			var c = com_watabou_geom_polygons_PolyCore.center(b);
			var v1 = (this.center == null ? this.center = com_watabou_geom_polygons_PolyCore.centroid(this.shape) : this.center).subtract(c);
			v1.normalize(dist);
			var mask = com_watabou_geom_polygons_PolyTransform.translate(this.shape, v1.x, v1.y);
			var res = com_watabou_geom_polygons_PolyBool.and(b, mask);
			if (res != null && res.length >= 3) {
				plans[i] = res;
			}
		}
	}
	, spawnTrees: function () {
		var trees = [];
		var greenery = this.group.district.greenery;
		if (!this.group.urban) {
			greenery *= 0.2;
		}
		if (this.courtyard != null) {
			var _g = 0;
			var _g1 = this.courtyard;
			while (_g < _g1.length) {
				var poly = _g1[_g];
				++_g;
				var b = com_watabou_mfcg_model_Tree.fillArea(poly, greenery);
				var _g2 = 0;
				while (_g2 < b.length) {
					var e = b[_g2];
					++_g2;
					trees.push(e);
				}
			}
		}
		return trees;
	}
	, area: function (poly) {
		if (this.cacheArea.h.__keys__[poly.__id__] != null) {
			return this.cacheArea.h[poly.__id__];
		} else {
			var this1 = this.cacheArea;
			var v = com_watabou_geom_polygons_PolyCore.area(poly);
			this1.set(poly, v);
			return v;
		}
	}
	, obb: function (poly) {
		if (this.cacheOBB.h.__keys__[poly.__id__] != null) {
			return this.cacheOBB.h[poly.__id__];
		} else {
			var this1 = this.cacheOBB;
			var v = com_watabou_geom_polygons_PolyBounds.obb(poly);
			this1.set(poly, v);
			return v;
		}
	}
	, __class__: com_watabou_mfcg_model_blocks_Block
};
var com_watabou_mfcg_model_blocks_TwistedBlock = function () { };
$hxClasses["com.watabou.mfcg.model.blocks.TwistedBlock"] = com_watabou_mfcg_model_blocks_TwistedBlock;
com_watabou_mfcg_model_blocks_TwistedBlock.__name__ = "com.watabou.mfcg.model.blocks.TwistedBlock";
com_watabou_mfcg_model_blocks_TwistedBlock.createLots = function (block, params, filterLots) {
	var variance = Math.max(params.sizeChaos * 4, 1.2);
	var bisector = new com_watabou_mfcg_utils_Bisector(block.shape, params.minSq, variance);
	bisector.minTurnOffset = 0.5;
	var lots = bisector.partition();
	lots = block.filterInner(lots);
	if (filterLots) {
		var _g = [];
		var _g1 = 0;
		var _g2 = lots;
		while (_g1 < _g2.length) {
			var v = _g2[_g1];
			++_g1;
			var obb;
			if (block.cacheOBB.h.__keys__[v.__id__] != null) {
				obb = block.cacheOBB.h[v.__id__];
			} else {
				var this1 = block.cacheOBB;
				var v1 = com_watabou_geom_polygons_PolyBounds.obb(v);
				this1.set(v, v1);
				obb = v1;
			}
			var a = openfl_geom_Point.distance(obb[0], obb[1]);
			var b = openfl_geom_Point.distance(obb[1], obb[2]);
			var lots1;
			if (a >= 1.2 && b >= 1.2) {
				var lots2;
				if (block.cacheArea.h.__keys__[v.__id__] != null) {
					lots2 = block.cacheArea.h[v.__id__];
				} else {
					var this2 = block.cacheArea;
					var v2 = com_watabou_geom_polygons_PolyCore.area(v);
					this2.set(v, v2);
					lots2 = v2;
				}
				lots1 = lots2 / (a * b) > 0.5;
			} else {
				lots1 = false;
			}
			if (lots1) {
				_g.push(v);
			}
		}
		lots = _g;
	}
	return lots;
};
var com_watabou_mfcg_model_blocks_VoronoiBlock = function () { };
$hxClasses["com.watabou.mfcg.model.blocks.VoronoiBlock"] = com_watabou_mfcg_model_blocks_VoronoiBlock;
com_watabou_mfcg_model_blocks_VoronoiBlock.__name__ = "com.watabou.mfcg.model.blocks.VoronoiBlock";
com_watabou_mfcg_model_blocks_VoronoiBlock.createLots = function (block, params, filterLots) {
	var seeds = [];
	var corners = [];
	var addSeed = function (p, corner) {
		if (corner == null) {
			corner = false;
		}
		var _g = 0;
		while (_g < seeds.length) {
			var p1 = seeds[_g];
			++_g;
			if (openfl_geom_Point.distance(p, p1) < 1.2) {
				return;
			}
		}
		seeds.push(p);
		if (corner) {
			corners.push(p);
		}
	};
	var step = params.minFront;
	var shape = block.shape;
	var len = shape.length;
	var b = shape[len - 1];
	var _g = 0;
	var _g1 = len;
	while (_g < _g1) {
		var i = _g++;
		var a = b;
		b = shape[i];
		addSeed(a, true);
		var n = Math.round(openfl_geom_Point.distance(a, b) / step);
		var _g2 = 1;
		var _g3 = n;
		while (_g2 < _g3) {
			var j = _g2++;
			var oo = 2 * (j / n - 0.5);
			var o = j - oo * ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) * params.sizeChaos;
			addSeed(com_watabou_geom_GeomUtils.lerp(a, b, o / n));
		}
	}
	var circle = com_watabou_geom_polygons_PolyBounds.circle(shape);
	var d = circle.r * 2;
	var cx = circle.c.x;
	var cy = circle.c.y;
	var _g = [];
	var p = openfl_geom_Point.polar(d, Math.PI * 0 / 3);
	_g.push(new openfl_geom_Point(p.x + cx, p.y + cy));
	var p = openfl_geom_Point.polar(d, Math.PI / 3);
	_g.push(new openfl_geom_Point(p.x + cx, p.y + cy));
	var p = openfl_geom_Point.polar(d, Math.PI * 2 / 3);
	_g.push(new openfl_geom_Point(p.x + cx, p.y + cy));
	var p = openfl_geom_Point.polar(d, Math.PI * 3 / 3);
	_g.push(new openfl_geom_Point(p.x + cx, p.y + cy));
	var p = openfl_geom_Point.polar(d, Math.PI * 4 / 3);
	_g.push(new openfl_geom_Point(p.x + cx, p.y + cy));
	var p = openfl_geom_Point.polar(d, Math.PI * 5 / 3);
	_g.push(new openfl_geom_Point(p.x + cx, p.y + cy));
	var bounds = _g;
	var sectors = new com_watabou_geom_Delaunator(seeds.concat(bounds)).getVoronoi();
	var map = sectors;
	var _g3_map = map;
	var _g3_keys = map.keys();
	while (_g3_keys.hasNext()) {
		var key = _g3_keys.next();
		var _g4_value = _g3_map.get(key);
		var _g4_key = key;
		var seed = _g4_key;
		var sector = _g4_value;
		var v = com_watabou_geom_polygons_PolyBool.and(sector, shape);
		sectors.set(seed, v);
	}
	var lots = [];
	var antiShape = com_watabou_utils_ArrayExtender.revert(shape);
	var map = sectors;
	var _g3_map = map;
	var _g3_keys = map.keys();
	while (_g3_keys.hasNext()) {
		var key = _g3_keys.next();
		var _g4_value = _g3_map.get(key);
		var _g4_key = key;
		var seed = _g4_key;
		var sector = _g4_value;
		if (sector != null) {
			var sectorArea;
			if (block.cacheArea.h.__keys__[sector.__id__] != null) {
				sectorArea = block.cacheArea.h[sector.__id__];
			} else {
				var this1 = block.cacheArea;
				var v = com_watabou_geom_polygons_PolyCore.area(sector);
				this1.set(sector, v);
				sectorArea = v;
			}
			if (isNaN(sectorArea)) {
				continue;
			}
			var targetArea = params.minSq * Math.pow(2, params.sizeChaos * (2 * ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) - 1));
			if (corners.indexOf(seed) != -1) {
				targetArea /= 2;
			}
			if (sectorArea < targetArea) {
				if (!filterLots || sectorArea > params.minSq / 4) {
					if (!filterLots || sector.length > 3) {
						lots.push(sector);
					}
				}
			} else {
				var dx = cx - seed.x;
				var dy = cy - seed.y;
				var d = Math.sqrt(dx * dx + dy * dy);
				var clipped = null;
				var _g = 1;
				while (_g < 10) {
					var i = _g++;
					var ofs = 1 - i / 10;
					var clip = com_watabou_geom_polygons_PolyTransform.translate(antiShape, dx * ofs, dy * ofs);
					clipped = com_watabou_geom_polygons_PolyBool.and(sector, clip, true);
					if (clipped == null) {
						clipped = sector;
					}
					var tmp;
					if (!(d * ofs < step)) {
						var tmp1;
						if (block.cacheArea.h.__keys__[clipped.__id__] != null) {
							tmp1 = block.cacheArea.h[clipped.__id__];
						} else {
							var this2 = block.cacheArea;
							var v1 = com_watabou_geom_polygons_PolyCore.area(clipped);
							this2.set(clipped, v1);
							tmp1 = v1;
						}
						tmp = tmp1 < targetArea;
					} else {
						tmp = true;
					}
					if (tmp) {
						break;
					}
				}
				if (!filterLots || clipped.length > 3) {
					lots.push(clipped);
				}
			}
		}
	}
	return lots;
};
com_watabou_mfcg_model_blocks_VoronoiBlock.merge = function (p1, p2) {
	var n1 = p1.length;
	var n2 = p2.length;
	var _g = 0;
	var _g1 = n1;
	while (_g < _g1) {
		var i = _g++;
		var j = p2.indexOf(p1[i]);
		if (j != -1) {
			var i1 = i;
			var i11 = (i1 + n1 - 1) % n1;
			var j1 = (j + 1) % n2;
			if (p1[i11] == p2[j1]) {
				i1 = i11;
				j = j1;
			}
			return p2.slice(j).concat(p2.slice(0, j)).concat(p1.slice(i1 + 2)).concat(p1.slice(0, i1));
		}
	}
	return null;
};
var com_watabou_mfcg_model_wards_Ward = function (model, patch) {
	this.model = model;
	this.patch = patch;
	patch.ward = this;
};
$hxClasses["com.watabou.mfcg.model.wards.Ward"] = com_watabou_mfcg_model_wards_Ward;
com_watabou_mfcg_model_wards_Ward.__name__ = "com.watabou.mfcg.model.wards.Ward";
com_watabou_mfcg_model_wards_Ward.inset = function (poly, dist, corners) {
	var result = com_watabou_mfcg_utils_PolyUtils.inset(poly, dist);
	if (result == null) {
		return null;
	}
	var len = corners.length;
	var _g = 0;
	var _g1 = len;
	while (_g < _g1) {
		var i = _g++;
		var d = corners[i];
		var j = (i + len - 1) % len;
		if (d > dist[i] && d > dist[j]) {
			var p = poly[i];
			var circ = com_watabou_geom_polygons_PolyCreate.regular(9, d, (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647);
			com_watabou_geom_polygons_PolyTransform.asTranslate(circ, p.x, p.y);
			var result1 = com_watabou_geom_polygons_PolyBool.and(result, com_watabou_utils_ArrayExtender.revert(circ), true);
			if (result1 != null) {
				result = result1;
			}
		}
	}
	return result;
};
com_watabou_mfcg_model_wards_Ward.prototype = {
	createGeometry: function () {
	}
	, spawnTrees: function () {
		return null;
	}
	, getAvailable: function () {
		var len = this.patch.shape.length;
		var corners = [];
		var _g_startEdge = this.patch.face.halfEdge;
		var _g_curEdge = _g_startEdge;
		var _g__hasNext = true;
		while (_g__hasNext) {
			var result = _g_curEdge;
			_g_curEdge = _g_curEdge.next;
			_g__hasNext = _g_curEdge != _g_startEdge;
			var e = result;
			var v = e.origin;
			var radius = 0.0;
			var _g = 0;
			var _g1 = this.model.walls;
			while (_g < _g1.length) {
				var wall = _g1[_g];
				++_g;
				radius = Math.max(radius, wall.getTowerRadius(v));
			}
			var _g2 = 0;
			var _g3 = this.model.canals;
			while (_g2 < _g3.length) {
				var canal = _g3[_g2];
				++_g2;
				if (com_watabou_geom_EdgeChain.edgeByOrigin(canal.course, v) != null) {
					radius = Math.max(radius, canal.width);
				}
			}
			corners.push(radius);
		}
		var _g = [];
		var _g1_startEdge = this.patch.face.halfEdge;
		var _g1_curEdge = _g1_startEdge;
		var _g1__hasNext = true;
		while (_g1__hasNext) {
			var result = _g1_curEdge;
			_g1_curEdge = _g1_curEdge.next;
			_g1__hasNext = _g1_curEdge != _g1_startEdge;
			var edge = result;
			_g.push(edge);
		}
		var edges = _g;
		var _g = [];
		var _g1 = 0;
		var _g2 = len;
		while (_g1 < _g2) {
			var i = _g1++;
			var edge = edges[i];
			var _g3 = 0;
			var _g4 = this.model.canals;
			while (_g3 < _g4.length) {
				var canal = _g4[_g3];
				++_g3;
				if (com_watabou_geom_EdgeChain.edgeByOrigin(canal.course, edge.origin) != null) {
					corners[i] = canal.width / 2 + 1.2;
					if (edge.origin == canal.course[0].origin) {
						corners[i] += 1.2;
					}
				}
			}
			if (edge.data == null) {
				_g.push((edge.twin.face.data == this.model.plaza ? 2.0 : 1.2) / 2);
			} else {
				var _g5 = edge.data;
				var tmp;
				switch (_g5._hx_index) {
					case 1:
						tmp = this.patch.landing ? 2.0 : 1.2;
						break;
					case 2:
						tmp = 1.;
						break;
					case 3:
						tmp = com_watabou_mfcg_model_CurtainWall.THICKNESS / 2 + 1.2;
						break;
					case 4:
						var canal1 = _g5.c;
						tmp = canal1.width / 2 + 1.2;
						break;
					default:
						tmp = 0.0;
				}
				_g.push(tmp);
			}
		}
		var insetDist = _g;
		return com_watabou_mfcg_model_wards_Ward.inset(this.patch.shape, insetDist, corners);
	}
	, getLabel: function () {
		if (this.patch.district != null) {
			return this.patch.district.name;
		} else {
			return null;
		}
	}
	, getColor: function () {
		if (this.patch.district != null) {
			return this.patch.district.color;
		} else {
			return 0;
		}
	}
	, onContext: function (context, x, y) {
	}
	, __class__: com_watabou_mfcg_model_wards_Ward
};
var com_watabou_mfcg_model_wards_Alleys = function (model, patch) {
	com_watabou_mfcg_model_wards_Ward.call(this, model, patch);
};
$hxClasses["com.watabou.mfcg.model.wards.Alleys"] = com_watabou_mfcg_model_wards_Alleys;
com_watabou_mfcg_model_wards_Alleys.__name__ = "com.watabou.mfcg.model.wards.Alleys";
com_watabou_mfcg_model_wards_Alleys.__super__ = com_watabou_mfcg_model_wards_Ward;
com_watabou_mfcg_model_wards_Alleys.prototype = $extend(com_watabou_mfcg_model_wards_Ward.prototype, {
	createGeometry: function () {
		if (this.group.core == this.patch) {
			this.group.createGeometry();
		}
		this.trees = null;
	}
	, spawnTrees: function () {
		if (this.group.core == this.patch && this.trees == null) {
			this.trees = [];
			var _g = 0;
			var _g1 = this.group.blocks;
			while (_g < _g1.length) {
				var block = _g1[_g];
				++_g;
				var a = this.trees;
				var b = block.spawnTrees();
				var _g2 = 0;
				while (_g2 < b.length) {
					var e = b[_g2];
					++_g2;
					a.push(e);
				}
			}
		}
		return this.trees;
	}
	, onContext: function (context, x, y) {
		var mode = com_watabou_system_State.get("display_mode", "Lots");
		if (mode == "Block") {
			return;
		}
		var p = new openfl_geom_Point(x, y);
		var _g = 0;
		var _g1 = this.group.blocks;
		while (_g < _g1.length) {
			var block = _g1[_g];
			++_g;
			if (com_watabou_geom_polygons_PolyBounds.containsPoint(block.shape, p)) {
				var polies;
				switch (mode) {
					case "Complex":
						polies = block.buildings;
						break;
					case "Simple":
						polies = block.rects;
						break;
					default:
						polies = block.lots;
				}
				var _g2 = 0;
				while (_g2 < polies.length) {
					var poly = [polies[_g2]];
					++_g2;
					if (com_watabou_geom_polygons_PolyBounds.containsPoint(poly[0], p)) {
						var id = [this.model.bp.seed];
						id[0] += this.model.patches.indexOf(this.patch);
						id[0] += this.group.blocks.indexOf(block);
						id[0] += polies.indexOf(poly[0]);
						context.addItem("Open in PM", (function (id, poly) {
							return function () {
								com_watabou_mfcg_model_wards_Mansion.openInPM(poly[0], id[0]);
							};
						})(id, poly));
						break;
					}
				}
				return;
			}
		}
	}
	, __class__: com_watabou_mfcg_model_wards_Alleys
});
var com_watabou_mfcg_model_wards_Castle = function (model, patch) {
	com_watabou_mfcg_model_wards_Ward.call(this, model, patch);
	var _g = [];
	var _g1 = 0;
	var _g2 = patch.shape;
	while (_g1 < _g2.length) {
		var v = _g2[_g1];
		++_g1;
		if (com_watabou_utils_ArrayExtender.some(model.patchesByVertex(model.dcel.vertices.h[v.__id__]), function (p) {
			return !p.withinCity;
		})) {
			_g.push(v);
		}
	}
	this.wall = new com_watabou_mfcg_model_CurtainWall(true, model, [patch], _g);
	this.adjustShape(model);
};
$hxClasses["com.watabou.mfcg.model.wards.Castle"] = com_watabou_mfcg_model_wards_Castle;
com_watabou_mfcg_model_wards_Castle.__name__ = "com.watabou.mfcg.model.wards.Castle";
com_watabou_mfcg_model_wards_Castle.__super__ = com_watabou_mfcg_model_wards_Ward;
com_watabou_mfcg_model_wards_Castle.prototype = $extend(com_watabou_mfcg_model_wards_Ward.prototype, {
	adjustShape: function (model) {
		var shape = this.patch.shape;
		var len = shape.length;
		var c = com_watabou_geom_polygons_PolyCore.centroid(shape);
		var minR = Infinity;
		var maxR = 0.0;
		var _g = 0;
		while (_g < shape.length) {
			var p = shape[_g];
			++_g;
			var r = openfl_geom_Point.distance(p, c);
			if (minR > r) {
				minR = r;
			}
			if (maxR < r) {
				maxR = r;
			}
		}
		while (minR < 10) {
			haxe_Log.trace("Adjusting the citadel size...", { fileName: "Source/com/watabou/mfcg/model/wards/Castle.hx", lineNumber: 56, className: "com.watabou.mfcg.model.wards.Castle", methodName: "adjustShape" });
			var radius = Math.max(15, maxR) * 2;
			var map = model.dcel.vertices;
			var _g1_map = map;
			var _g1_keys = map.keys();
			while (_g1_keys.hasNext()) {
				var key = _g1_keys.next();
				var _g2_value = _g1_map.get(key);
				var _g2_key = key;
				var p = _g2_key;
				var v = _g2_value;
				var l = openfl_geom_Point.distance(p, c);
				if (l < radius) {
					var p1 = p.subtract(c);
					var f = Math.pow(openfl_geom_Point.distance(p, c) / radius, -0.25);
					p1.x *= f;
					p1.y *= f;
					p1.x += c.x;
					p1.y += c.y;
					com_watabou_utils_PointExtender.set(p, p1);
				}
			}
			minR = Infinity;
			maxR = 0.0;
			var _g = 0;
			while (_g < shape.length) {
				var p2 = shape[_g];
				++_g;
				var r = openfl_geom_Point.distance(p2, c);
				if (minR > r) {
					minR = r;
				}
				if (maxR < r) {
					maxR = r;
				}
			}
		}
		var vGate = this.wall.gates[0];
		var gate = this.wall.gates[0].point;
		var fixed;
		if (vGate.edges.length == 2) {
			var i = shape.indexOf(gate);
			var fixed1;
			if (i != -1) {
				var len1 = shape.length;
				fixed1 = shape[(i + len1 - 1) % len1];
			} else {
				fixed1 = null;
			}
			var i = shape.indexOf(gate);
			fixed = [gate, fixed1, i != -1 ? shape[(i + 1) % shape.length] : null];
		} else {
			fixed = [gate];
		}
		var c = com_watabou_geom_polygons_PolyCore.compactness(shape);
		while (c < 0.75) {
			var _g = [];
			var _g1 = 0;
			var _g2 = len;
			while (_g1 < _g2) {
				var i = _g1++;
				var v1 = shape[i];
				if (fixed.indexOf(v1) == -1) {
					var v0 = shape[(i + len - 1) % len];
					var v2 = shape[(i + 1) % len];
					var v = v2.subtract(v0);
					var d = (v.x * v1.y - v.y * v1.x - v2.x * v0.y + v2.y * v0.x) / v.get_length();
					var tmp = com_watabou_geom_GeomUtils.lerp(v0, v2);
					var p = new openfl_geom_Point(-v.y, v.x);
					var length = d;
					if (length == null) {
						length = 1;
					}
					p = p.clone();
					p.normalize(length);
					_g.push(tmp.add(p));
				} else {
					_g.push(v1);
				}
			}
			var sh = _g;
			var _g3 = 0;
			var _g4 = len;
			while (_g3 < _g4) {
				var i1 = _g3++;
				com_watabou_utils_PointExtender.set(shape[i1], sh[i1]);
			}
			var c1 = com_watabou_geom_polygons_PolyCore.compactness(shape);
			if (c1 == c) {
				throw new openfl_errors_Error("Bad citadel shape!");
			} else {
				c = c1;
			}
		}
	}
	, createGeometry: function () {
		com_watabou_utils_Random.restore(this.patch.seed);
		var block = com_watabou_geom_polygons_PolyCut.shrinkEq(this.patch.shape, com_watabou_mfcg_model_CurtainWall.THICKNESS + 2.0);
		var keep = com_watabou_geom_polygons_PolyBounds.lira(block);
		this.building = com_watabou_mfcg_model_Building.create(keep, com_watabou_geom_polygons_PolyCore.area(this.patch.shape) / 25, null, null, 0.4);
		if (this.building == null) {
			this.building = keep;
		}
	}
	, getLabel: function () {
		return "Castle";
	}
	, onContext: function (context, x, y) {
		var _gthis = this;
		if (com_watabou_geom_polygons_PolyBounds.containsPoint(this.building, new openfl_geom_Point(x, y))) {
			var id = this.model.bp.seed;
			id += this.model.patches.indexOf(this.patch);
			context.addItem("Open in PM", function () {
				com_watabou_mfcg_model_wards_Mansion.openInPM(_gthis.building, id);
			});
		}
	}
	, __class__: com_watabou_mfcg_model_wards_Castle
});
var com_watabou_mfcg_model_wards_Cathedral = function (model, patch) {
	com_watabou_mfcg_model_wards_Ward.call(this, model, patch);
};
$hxClasses["com.watabou.mfcg.model.wards.Cathedral"] = com_watabou_mfcg_model_wards_Cathedral;
com_watabou_mfcg_model_wards_Cathedral.__name__ = "com.watabou.mfcg.model.wards.Cathedral";
com_watabou_mfcg_model_wards_Cathedral.__super__ = com_watabou_mfcg_model_wards_Ward;
com_watabou_mfcg_model_wards_Cathedral.prototype = $extend(com_watabou_mfcg_model_wards_Ward.prototype, {
	createGeometry: function () {
		com_watabou_utils_Random.restore(this.patch.seed);
		var block = this.getAvailable();
		if (block == null) {
			this.building = [];
		} else {
			var rect = com_watabou_geom_polygons_PolyBounds.lira(block);
			var b = com_watabou_mfcg_model_Building.create(rect, 20, false, true, 0.2);
			this.building = [b != null ? b : rect];
		}
	}
	, onContext: function (context, x, y) {
		var p = new openfl_geom_Point(x, y);
		var _g = 0;
		var _g1 = this.building;
		while (_g < _g1.length) {
			var b = [_g1[_g]];
			++_g;
			if (com_watabou_geom_polygons_PolyBounds.containsPoint(b[0], p)) {
				var id = [this.model.bp.seed];
				id[0] += this.model.patches.indexOf(this.patch);
				context.addItem("Open in PM", (function (id, b) {
					return function () {
						com_watabou_mfcg_model_wards_Mansion.openInPM(b[0], id[0]);
					};
				})(id, b));
			}
		}
	}
	, __class__: com_watabou_mfcg_model_wards_Cathedral
});
var com_watabou_mfcg_model_wards_Farm = function (model, patch) {
	com_watabou_mfcg_model_wards_Ward.call(this, model, patch);
};
$hxClasses["com.watabou.mfcg.model.wards.Farm"] = com_watabou_mfcg_model_wards_Farm;
com_watabou_mfcg_model_wards_Farm.__name__ = "com.watabou.mfcg.model.wards.Farm";
com_watabou_mfcg_model_wards_Farm.__super__ = com_watabou_mfcg_model_wards_Ward;
com_watabou_mfcg_model_wards_Farm.prototype = $extend(com_watabou_mfcg_model_wards_Ward.prototype, {
	getAvailable: function () {
		var len = this.patch.shape.length;
		var corners = [];
		var _g_startEdge = this.patch.face.halfEdge;
		var _g_curEdge = _g_startEdge;
		var _g__hasNext = true;
		while (_g__hasNext) {
			var result = _g_curEdge;
			_g_curEdge = _g_curEdge.next;
			_g__hasNext = _g_curEdge != _g_startEdge;
			var e = result;
			var radius = 0.0;
			var _g = 0;
			var _g1 = this.model.walls;
			while (_g < _g1.length) {
				var wall = _g1[_g];
				++_g;
				radius = Math.max(radius, wall.getTowerRadius(e.origin));
			}
			corners.push(radius);
		}
		var _g = [];
		var _g1_startEdge = this.patch.face.halfEdge;
		var _g1_curEdge = _g1_startEdge;
		var _g1__hasNext = true;
		while (_g1__hasNext) {
			var result = _g1_curEdge;
			_g1_curEdge = _g1_curEdge.next;
			_g1__hasNext = _g1_curEdge != _g1_startEdge;
			var edge = result;
			_g.push(edge);
		}
		var edges = _g;
		var _g = [];
		var _g1 = 0;
		var _g2 = len;
		while (_g1 < _g2) {
			var i = _g1++;
			var edge = edges[i];
			var _g3 = 0;
			var _g4 = this.model.canals;
			while (_g3 < _g4.length) {
				var canal = _g4[_g3];
				++_g3;
				if (com_watabou_geom_EdgeChain.edgeByOrigin(canal.course, edge.origin) != null) {
					corners[i] = canal.width / 2 + 1.2;
					if (edge.origin == canal.course[0].origin) {
						corners[i] += 1.2;
					}
				}
			}
			if (edge.data == null) {
				_g.push(edge.twin != null && ((edge.twin.face.data.ward) instanceof com_watabou_mfcg_model_wards_Farm) ? 1. : 0.0);
			} else {
				var _g5 = edge.data;
				var tmp;
				switch (_g5._hx_index) {
					case 2:
						tmp = 2.0 + com_watabou_mfcg_mapping_Style.strokeNormal * 2;
						break;
					case 3:
						tmp = com_watabou_mfcg_model_CurtainWall.THICKNESS * 2;
						break;
					case 4:
						var canal1 = _g5.c;
						tmp = canal1.width / 2 + 1.2;
						break;
					default:
						tmp = 2.0;
				}
				_g.push(tmp);
			}
		}
		var insetDist = _g;
		return com_watabou_mfcg_model_wards_Ward.inset(this.patch.shape, insetDist, corners);
	}
	, createGeometry: function () {
		var _gthis = this;
		com_watabou_utils_Random.restore(this.patch.seed);
		var field = this.getAvailable();
		this.furrows = [];
		this.edges = [];
		this.subPlots = this.splitField(field);
		var outer = [];
		var e = this.patch.face.halfEdge;
		while (true) {
			if (e.twin != null && js_Boot.getClass(e.twin.face.data.ward) == com_watabou_mfcg_model_wards_Ward) {
				outer.push(e);
			}
			e = e.next;
			if (!(e != this.patch.face.halfEdge)) {
				break;
			}
		}
		if (outer.length > 0) {
			var _g = [];
			var _g1 = 0;
			var _g2 = this.subPlots;
			while (_g1 < _g2.length) {
				var v = _g2[_g1];
				++_g1;
				if ((function (lot) {
					var v1 = lot[lot.length - 1];
					var _g = 0;
					var _g1 = lot.length;
					while (_g < _g1) {
						var i = _g++;
						var v0 = v1;
						v1 = lot[i];
						var _g2 = 0;
						while (_g2 < outer.length) {
							var edge = outer[_g2];
							++_g2;
							var u0 = edge.origin.point;
							var u1 = edge.next.origin.point;
							if (com_watabou_geom_GeomUtils.converge(v0, v1, u0, u1)) {
								return false;
							}
						}
					}
					return true;
				})(v)) {
					_g.push(v);
				}
			}
			this.subPlots = _g;
		}
		var _g = 0;
		var _g1 = this.subPlots.length;
		while (_g < _g1) {
			var i = _g++;
			var sp = this.subPlots[i] = this.round(this.subPlots[i]);
			var obb = com_watabou_geom_polygons_PolyBounds.obb(sp);
			if (openfl_geom_Point.distance(obb[1], obb[0]) < openfl_geom_Point.distance(obb[2], obb[1]) == ((i & 1) == 0)) {
				obb.push(obb.shift());
			}
			var plotLength = openfl_geom_Point.distance(obb[0], obb[1 % obb.length]);
			var furrowWidth = com_watabou_mfcg_model_wards_Farm.MIN_FURROW * (1 + ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 3);
			var nFurrows = Math.ceil(plotLength / furrowWidth);
			var step = plotLength / nFurrows;
			var pos = step / 2;
			while (pos < plotLength) {
				var r = pos / plotLength;
				var a = com_watabou_geom_GeomUtils.lerp(obb[0], obb[1], r);
				var b = com_watabou_geom_GeomUtils.lerp(obb[3], obb[2], r);
				var p = com_watabou_geom_polygons_PolyCut.pierce(sp, a, b);
				while (p.length >= 2) {
					var start = p.shift();
					var end = p.shift();
					if (openfl_geom_Point.distance(start, end) > 1.2) {
						_gthis.furrows.push(new com_watabou_geom_Segment(start, end));
					}
				}
				pos += step;
			}
		}
		var _g = [];
		var _g1 = 0;
		var _g2 = this.subPlots;
		while (_g1 < _g2.length) {
			var plot = _g2[_g1];
			++_g1;
			var chance = 0.2;
			if (chance == null) {
				chance = 0.5;
			}
			if ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance) {
				_g.push(this.getHousing(plot));
			}
		}
		this.buildings = _g;
		this.trees = null;
	}
	, splitField: function (field) {
		if (com_watabou_geom_polygons_PolyCore.area(field) < com_watabou_mfcg_model_wards_Farm.MIN_SUBPLOT * (1 + Math.abs(((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 2 - 1))) {
			return [field];
		} else {
			var obb = com_watabou_geom_polygons_PolyBounds.obb(field);
			var long = openfl_geom_Point.distance(obb[1], obb[0]) > openfl_geom_Point.distance(obb[2], obb[1]) ? 0 : 1;
			var ratio = 0.5 + 0.2 * (((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 3 * 2 - 1);
			var chance = 0.5;
			if (chance == null) {
				chance = 0.5;
			}
			var angle = Math.PI / 2 + ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance ? 0 : Math.PI / 8 * (((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 3 * 2 - 1));
			var p1 = com_watabou_geom_GeomUtils.lerp(obb[long], obb[long + 1], ratio);
			var p = obb[long < obb.length - 1 ? long + 1 : 0].subtract(obb[long]);
			var sin = Math.sin(angle);
			var cos = Math.cos(angle);
			var p2 = p1.add(new openfl_geom_Point(p.x * cos - p.y * sin, p.y * cos + p.x * sin));
			var halves = com_watabou_geom_polygons_PolyCut.cut(field, p1, p2, 2.0);
			var subs = [];
			var _g = 0;
			while (_g < halves.length) {
				var half = halves[_g];
				++_g;
				var b = this.splitField(half);
				var _g1 = 0;
				while (_g1 < b.length) {
					var e = b[_g1];
					++_g1;
					subs.push(e);
				}
			}
			return subs;
		}
	}
	, round: function (poly) {
		var result = [];
		var len = poly.length;
		var _g = 0;
		var _g1 = len;
		while (_g < _g1) {
			var i = _g++;
			var v0 = poly[i];
			var v1 = poly[(i + 1) % len];
			var d = openfl_geom_Point.distance(v0, v1);
			if (d < com_watabou_mfcg_model_wards_Farm.MIN_FURROW * 2) {
				result.push(com_watabou_geom_GeomUtils.lerp(v0, v1));
			} else {
				result.push(com_watabou_geom_GeomUtils.lerp(v0, v1, com_watabou_mfcg_model_wards_Farm.MIN_FURROW / d));
				result.push(com_watabou_geom_GeomUtils.lerp(v1, v0, com_watabou_mfcg_model_wards_Farm.MIN_FURROW / d));
			}
		}
		return result;
	}
	, getHousing: function (plot) {
		var length = 4 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647;
		var depth = 2 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647;
		var rect = com_watabou_geom_polygons_PolyCreate.rect(length, depth);
		var edge = com_watabou_geom_polygons_PolyAccess.longest(plot);
		var p = plot[edge < plot.length - 1 ? edge + 1 : 0].subtract(plot[edge]);
		p = p.clone();
		p.normalize(1);
		var edgeVector = p;
		var from = plot[edge];
		var to = plot[(edge + 1) % plot.length];
		var pos;
		if ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < 0.5) {
			var f = length / 2;
			pos = from.add(new openfl_geom_Point(edgeVector.x * f, edgeVector.y * f));
		} else {
			var f = length / 2;
			pos = to.subtract(new openfl_geom_Point(edgeVector.x * f, edgeVector.y * f));
		}
		var p = new openfl_geom_Point(-edgeVector.y, edgeVector.x);
		var f = depth / 2;
		var q = new openfl_geom_Point(p.x * f, p.y * f);
		pos.x += q.x;
		pos.y += q.y;
		com_watabou_geom_polygons_PolyTransform.asRotateYX(rect, edgeVector.y, edgeVector.x);
		com_watabou_geom_polygons_PolyTransform.asAdd(rect, pos);
		var b = com_watabou_mfcg_model_Building.create(rect, 4 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647, null, null, 0.4);
		if (b != null) {
			return b;
		} else {
			return rect;
		}
	}
	, spawnTrees: function () {
		if (this.trees == null) {
			this.trees = [];
			var r = Math.max(this.model.maxx - this.model.minx, this.model.maxy - this.model.miny) * (((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 3);
			var _g = 0;
			var _g1 = this.subPlots;
			while (_g < _g1.length) {
				var sp = _g1[_g];
				++_g;
				var _g2 = 0;
				var _g3 = sp.length;
				while (_g2 < _g3) {
					var i = _g2++;
					var p0 = sp[i];
					var p1 = sp[(i + 1) % sp.length];
					var c = com_watabou_geom_GeomUtils.lerp(p0, p1);
					var density = 1 - c.get_length() / r;
					var chance = density;
					if (chance == null) {
						chance = 0.5;
					}
					if ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance) {
						var n = Math.ceil(((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 3 * openfl_geom_Point.distance(p0, p1) / 4.);
						var _g4 = 0;
						var _g5 = n + 1;
						while (_g4 < _g5) {
							var j = _g4++;
							var p = com_watabou_geom_GeomUtils.lerp(p0, p1, j / n);
							this.trees.push(new com_watabou_mfcg_model_Tree(p));
						}
					}
				}
			}
		}
		return this.trees;
	}
	, getLabel: function () {
		return "Farm";
	}
	, onContext: function (context, x, y) {
		if (com_watabou_system_State.get("display_mode", "Lots") == "Block") {
			return;
		}
		var p = new openfl_geom_Point(x, y);
		var _g = 0;
		var _g1 = this.buildings;
		while (_g < _g1.length) {
			var b = [_g1[_g]];
			++_g;
			if (com_watabou_geom_polygons_PolyBounds.containsPoint(b[0], p)) {
				var id = [this.model.bp.seed];
				id[0] += this.model.patches.indexOf(this.patch);
				id[0] += this.buildings.indexOf(b[0]);
				context.addItem("Open in PM", (function (id, b) {
					return function () {
						com_watabou_mfcg_model_wards_Mansion.openInPM(b[0], id[0]);
					};
				})(id, b));
			}
		}
	}
	, __class__: com_watabou_mfcg_model_wards_Farm
});
var com_watabou_mfcg_model_wards_Harbour = function (model, patch) {
	com_watabou_mfcg_model_wards_Ward.call(this, model, patch);
};
$hxClasses["com.watabou.mfcg.model.wards.Harbour"] = com_watabou_mfcg_model_wards_Harbour;
com_watabou_mfcg_model_wards_Harbour.__name__ = "com.watabou.mfcg.model.wards.Harbour";
com_watabou_mfcg_model_wards_Harbour.__super__ = com_watabou_mfcg_model_wards_Ward;
com_watabou_mfcg_model_wards_Harbour.prototype = $extend(com_watabou_mfcg_model_wards_Ward.prototype, {
	createGeometry: function () {
		var _g = [];
		var _g1 = 0;
		var _g2 = this.model.canals;
		while (_g1 < _g2.length) {
			var canal = _g2[_g1];
			++_g1;
			_g.push(canal.course[0].origin);
		}
		var mouths = _g;
		var segs = [];
		var edge = this.patch.face.halfEdge;
		while (true) {
			var n = this.model.getNeighbour(this.patch, edge.origin);
			if (n != null && n.landing) {
				var v1 = edge.origin.point;
				var v2 = edge.next.origin.point;
				if (mouths.indexOf(edge.origin) != -1) {
					segs.push(new com_watabou_geom_Segment(com_watabou_geom_GeomUtils.lerp(edge.origin.point, edge.next.origin.point, 0.5), v2));
				} else if (mouths.indexOf(edge.next.origin) != -1) {
					segs.push(new com_watabou_geom_Segment(v1, com_watabou_geom_GeomUtils.lerp(edge.origin.point, edge.next.origin.point, 0.5)));
				} else {
					segs.push(new com_watabou_geom_Segment(v1, v2));
				}
			}
			edge = edge.next;
			if (!(edge != this.patch.face.halfEdge)) {
				break;
			}
		}
		if (segs.length > 0) {
			var front = com_watabou_utils_ArrayExtender.max(segs, function (seg) {
				return openfl_geom_Point.distance(seg.start, seg.end);
			});
			var length = openfl_geom_Point.distance(front.start, front.end);
			var nPiers = length / 6 | 0;
			var width = (nPiers - 1) * 6;
			var pos = (1 - width / length) / 2;
			var step = width / (nPiers - 1) / length;
			var _g = [];
			var _g1 = 0;
			var _g2 = nPiers;
			while (_g1 < _g2) {
				var i = _g1++;
				var v1 = com_watabou_geom_GeomUtils.lerp(front.start, front.end, pos);
				var p = front.end.subtract(front.start);
				var p1 = new openfl_geom_Point(-p.y, p.x);
				var length = 8;
				if (length == null) {
					length = 1;
				}
				p1 = p1.clone();
				p1.normalize(length);
				var v2 = v1.add(p1);
				pos += step;
				_g.push([v1, v2]);
			}
			this.piers = _g;
		} else {
			this.piers = [];
		}
	}
	, getLabel: function () {
		return "Harbour";
	}
	, __class__: com_watabou_mfcg_model_wards_Harbour
});
var com_watabou_mfcg_model_wards_Mansion = function () { };
$hxClasses["com.watabou.mfcg.model.wards.Mansion"] = com_watabou_mfcg_model_wards_Mansion;
com_watabou_mfcg_model_wards_Mansion.__name__ = "com.watabou.mfcg.model.wards.Mansion";
com_watabou_mfcg_model_wards_Mansion.openInPM = function (poly, seed) {
	var rect = new com_watabou_mfcg_model_wards__$Mansion_Rect(poly);
	haxe_Log.trace(rect.len0, { fileName: "Source/com/watabou/mfcg/model/wards/Mansion.hx", lineNumber: 19, className: "com.watabou.mfcg.model.wards.Mansion", methodName: "openInPM", customParams: [rect.len1] });
	var width = com_watabou_utils_MathUtils.gatei(Math.round(rect.len0 / 1.5), 1, 13);
	var height = com_watabou_utils_MathUtils.gatei(Math.round(rect.len1 / 1.5), 1, 13);
	var request = new openfl_net_URLRequest(com_watabou_mfcg_model_wards_Mansion.MANSION_URL);
	request.data = { seed: seed, w: width, h: height, plan: com_watabou_mfcg_model_wards_Mansion.getPlan(rect, width, height), mode: "plan", from: "neighbourhoods" };
	openfl_Lib.navigateToURL(request, "mfcg2pm_" + seed);
};
com_watabou_mfcg_model_wards_Mansion.getPlan = function (rect, width, height) {
	var planStr = "";
	var d = 0;
	var c = 0;
	var _g = 0;
	var _g1 = height;
	while (_g < _g1) {
		var i = _g++;
		var _g2 = 0;
		var _g3 = width;
		while (_g2 < _g3) {
			var j = _g2++;
			var a = rect.o;
			var b = rect.v0;
			var t = (j + 0.5) / width;
			var a1 = new openfl_geom_Point(a.x + b.x * t, a.y + b.y * t);
			var b1 = rect.v1;
			var t1 = (height - 1 - i + 0.5) / height;
			var p = new openfl_geom_Point(a1.x + b1.x * t1, a1.y + b1.y * t1);
			var filled = com_watabou_geom_polygons_PolyBounds.containsPoint(rect.poly, p);
			if (filled) {
				d |= 1 << (c & 3);
			}
			if ((c & 3) == 3 || j == width - 1 && i == height - 1) {
				planStr += StringTools.hex(d);
				d = 0;
			}
			++c;
		}
	}
	return planStr;
};
var com_watabou_mfcg_model_wards__$Mansion_Rect = function (poly) {
	this.poly = poly;
	var rect = com_watabou_geom_polygons_PolyBounds.obb(poly);
	this.o = rect[0];
	this.v0 = rect[1].subtract(this.o);
	this.v1 = rect[3].subtract(this.o);
	this.len0 = this.v0.get_length();
	this.len1 = this.v1.get_length();
};
$hxClasses["com.watabou.mfcg.model.wards._Mansion.Rect"] = com_watabou_mfcg_model_wards__$Mansion_Rect;
com_watabou_mfcg_model_wards__$Mansion_Rect.__name__ = "com.watabou.mfcg.model.wards._Mansion.Rect";
com_watabou_mfcg_model_wards__$Mansion_Rect.prototype = {
	__class__: com_watabou_mfcg_model_wards__$Mansion_Rect
};
var com_watabou_mfcg_model_wards_Market = function (model, patch) {
	com_watabou_mfcg_model_wards_Ward.call(this, model, patch);
};
$hxClasses["com.watabou.mfcg.model.wards.Market"] = com_watabou_mfcg_model_wards_Market;
com_watabou_mfcg_model_wards_Market.__name__ = "com.watabou.mfcg.model.wards.Market";
com_watabou_mfcg_model_wards_Market.__super__ = com_watabou_mfcg_model_wards_Ward;
com_watabou_mfcg_model_wards_Market.prototype = $extend(com_watabou_mfcg_model_wards_Ward.prototype, {
	createGeometry: function () {
		com_watabou_utils_Random.restore(this.patch.seed);
		this.space = this.getAvailable();
		var chance = 0.6;
		if (chance == null) {
			chance = 0.5;
		}
		var statue = (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance;
		var offset;
		if (!statue) {
			var chance = 0.3;
			if (chance == null) {
				chance = 0.5;
			}
			offset = (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance;
		} else {
			offset = true;
		}
		var v0 = null;
		var v1 = null;
		if (statue || offset) {
			var longest = com_watabou_geom_polygons_PolyAccess.longest(this.space);
			v0 = this.space[longest];
			v1 = this.space[(longest + 1) % this.space.length];
		}
		if (statue) {
			this.monument = com_watabou_geom_polygons_PolyCreate.rect(1 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647, 1 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647);
			var poly = this.monument;
			var p = v1.subtract(v0);
			var angle = Math.atan2(p.y, p.x);
			com_watabou_geom_polygons_PolyTransform.asRotateYX(poly, Math.sin(angle), Math.cos(angle));
		} else {
			this.monument = com_watabou_geom_polygons_PolyCreate.regular(8, 1 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647);
		}
		var c = com_watabou_geom_polygons_PolyCore.centroid(this.space);
		if (offset) {
			var gravity = com_watabou_geom_GeomUtils.lerp(v0, v1);
			com_watabou_geom_polygons_PolyTransform.asAdd(this.monument, com_watabou_geom_GeomUtils.lerp(c, gravity, 0.2 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 * 0.4));
		} else {
			com_watabou_geom_polygons_PolyTransform.asAdd(this.monument, c);
		}
	}
	, getAvailable: function () {
		var len = this.patch.shape.length;
		var _g = [];
		var _g1 = 0;
		var _g2 = len;
		while (_g1 < _g2) {
			var i = _g1++;
			_g.push(0.0);
		}
		var corners = _g;
		var _g = [];
		var _g1_startEdge = this.patch.face.halfEdge;
		var _g1_curEdge = _g1_startEdge;
		var _g1__hasNext = true;
		while (_g1__hasNext) {
			var result = _g1_curEdge;
			_g1_curEdge = _g1_curEdge.next;
			_g1__hasNext = _g1_curEdge != _g1_startEdge;
			var edge = result;
			_g.push(edge);
		}
		var edges = _g;
		var _g = [];
		var _g1 = 0;
		var _g2 = len;
		while (_g1 < _g2) {
			var i = _g1++;
			var edge = edges[i];
			var _g3 = 0;
			var _g4 = this.model.canals;
			while (_g3 < _g4.length) {
				var canal = _g4[_g3];
				++_g3;
				if (com_watabou_geom_EdgeChain.edgeByOrigin(canal.course, edge.origin) != null) {
					corners[i] = canal.width / 2;
					if (edge.origin == canal.course[0].origin) {
						corners[i] += 1.2;
					}
				}
			}
			if (edge.data != null) {
				var _g5 = edge.data;
				var tmp;
				if (_g5._hx_index == 4) {
					var canal1 = _g5.c;
					tmp = canal1.width / 2;
				} else {
					tmp = 0.0;
				}
				_g.push(tmp);
			} else {
				_g.push(0.0);
			}
		}
		var insetDist = _g;
		return com_watabou_mfcg_model_wards_Ward.inset(this.patch.shape, insetDist, corners);
	}
	, __class__: com_watabou_mfcg_model_wards_Market
});
var com_watabou_mfcg_model_wards_Park = function (model, patch) {
	com_watabou_mfcg_model_wards_Ward.call(this, model, patch);
};
$hxClasses["com.watabou.mfcg.model.wards.Park"] = com_watabou_mfcg_model_wards_Park;
com_watabou_mfcg_model_wards_Park.__name__ = "com.watabou.mfcg.model.wards.Park";
com_watabou_mfcg_model_wards_Park.__super__ = com_watabou_mfcg_model_wards_Ward;
com_watabou_mfcg_model_wards_Park.prototype = $extend(com_watabou_mfcg_model_wards_Ward.prototype, {
	createGeometry: function () {
		var block = this.getAvailable();
		var points = [];
		var _g = 0;
		var _g1 = block.length;
		while (_g < _g1) {
			var i = _g++;
			var p = block[i];
			points.push(p);
			points.push(com_watabou_geom_GeomUtils.lerp(p, block[(i + 1) % block.length]));
		}
		this.green = com_watabou_geom_Chaikin.render(points, true, 3);
		this.trees = null;
	}
	, spawnTrees: function () {
		if (this.trees == null) {
			this.trees = com_watabou_mfcg_model_Tree.fillArea(this.getAvailable(), this.patch.district.greenery);
		}
		return this.trees;
	}
	, __class__: com_watabou_mfcg_model_wards_Park
});
var com_watabou_mfcg_model_wards_WardGroup = function (patches) {
	this.patches = patches;
	var _g = 0;
	while (_g < patches.length) {
		var patch = patches[_g];
		++_g;
		(js_Boot.__cast(patch.ward, com_watabou_mfcg_model_wards_Alleys)).group = this;
	}
	this.core = patches[0];
	this.model = this.core.ward.model;
	this.district = this.core.district;
	if (patches.length == 1) {
		this.shape = this.core.shape;
		var _g = [];
		var _g1_startEdge = this.core.face.halfEdge;
		var _g1_curEdge = _g1_startEdge;
		var _g1__hasNext = true;
		while (_g1__hasNext) {
			var result = _g1_curEdge;
			_g1_curEdge = _g1_curEdge.next;
			_g1__hasNext = _g1_curEdge != _g1_startEdge;
			var edge = result;
			_g.push(edge);
		}
		this.border = _g;
	} else if (patches.length == this.district.faces.length) {
		this.border = this.district.border;
		this.shape = com_watabou_geom_EdgeChain.toPoly(this.border);
	} else {
		var _g = new haxe_ds_ObjectMap();
		var _g1 = 0;
		while (_g1 < patches.length) {
			var p = patches[_g1];
			++_g1;
			_g.set(p.face, true);
		}
		this.border = com_watabou_geom_DCEL.circumference(null, _g);
		this.shape = com_watabou_geom_EdgeChain.toPoly(this.border);
	}
	this.inner = [];
	this.blockM = new haxe_ds_ObjectMap();
	var _g = 0;
	var _g1 = this.border;
	while (_g < _g1.length) {
		var edge = _g1[_g];
		++_g;
		var v = edge.origin;
		if (edge.face.data.withinWalls || this.isInnerVertex(v)) {
			this.inner.push(v);
			this.blockM.set(v.point, 1.0);
		} else {
			this.blockM.set(v.point, 9.0);
		}
	}
	this.urban = this.inner.length == this.border.length;
};
$hxClasses["com.watabou.mfcg.model.wards.WardGroup"] = com_watabou_mfcg_model_wards_WardGroup;
com_watabou_mfcg_model_wards_WardGroup.__name__ = "com.watabou.mfcg.model.wards.WardGroup";
com_watabou_mfcg_model_wards_WardGroup.prototype = {
	createGeometry: function () {
		com_watabou_utils_Random.restore(this.core.seed);
		var available = this.getAvailable();
		if (available == null) {
			haxe_Log.trace("Failed to calculate the available area", { fileName: "Source/com/watabou/mfcg/model/wards/WardGroup.hx", lineNumber: 84, className: "com.watabou.mfcg.model.wards.WardGroup", methodName: "createGeometry" });
			this.blocks = [];
		} else {
			var attemptsLeft = 20;
			while (true) {
				this.blocks = [];
				this.church = null;
				var available1 = available.slice();
				var tmp = com_watabou_geom_polygons_PolyCore.area(available1);
				var params = this.district.alleys;
				if (tmp > params.minSq * Math.pow(2, params.sizeChaos * (2 * ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) - 1)) * params.blockSize) {
					this.createAlleys(available1);
				} else {
					this.createBlock(available1);
				}
				if (!this.urban) {
					this.filter();
				}
				if (!(this.blocks.length == 0 && attemptsLeft-- > 0)) {
					break;
				}
			}
		}
		if (this.blocks.length == 0) {
			haxe_Log.trace("Failed to create a non-empty alleys group", { fileName: "Source/com/watabou/mfcg/model/wards/WardGroup.hx", lineNumber: 106, className: "com.watabou.mfcg.model.wards.WardGroup", methodName: "createGeometry" });
		}
	}
	, getAvailable: function () {
		var len = this.shape.length;
		var wide = 2.0;
		var narrow = 1.2;
		var corners = [];
		var _g = 0;
		var _g1 = this.border;
		while (_g < _g1.length) {
			var e = _g1[_g];
			++_g;
			var radius = 0.0;
			var _g2 = 0;
			var _g3 = this.model.walls;
			while (_g2 < _g3.length) {
				var wall = _g3[_g2];
				++_g2;
				var r = wall.getTowerRadius(e.origin);
				if (r > 0) {
					r += 1.2;
				}
				radius = Math.max(radius, r);
			}
			corners.push(radius);
		}
		var _g = [];
		var _g1 = 0;
		var _g2 = len;
		while (_g1 < _g2) {
			var i = _g1++;
			var edge = this.border[i];
			var _g3 = 0;
			var _g4 = this.model.canals;
			while (_g3 < _g4.length) {
				var canal = _g4[_g3];
				++_g3;
				if (com_watabou_geom_EdgeChain.edgeByOrigin(canal.course, edge.origin) != null) {
					corners[i] = canal.width / 2 + narrow;
					if (edge.origin == canal.course[0].origin) {
						corners[i] += narrow;
					}
				}
			}
			if (edge.data == null) {
				_g.push(edge.twin.face.data == this.model.plaza ? wide / 2 : narrow / 2);
			} else {
				var _g5 = edge.data;
				var tmp;
				switch (_g5._hx_index) {
					case 1:
						var patch = edge.face.data;
						tmp = patch.landing ? wide : narrow;
						break;
					case 2:
						tmp = wide / 2;
						break;
					case 3:
						tmp = com_watabou_mfcg_model_CurtainWall.THICKNESS / 2 + narrow;
						break;
					case 4:
						var canal1 = _g5.c;
						tmp = canal1.width / 2 + narrow;
						break;
					default:
						tmp = 0.0;
				}
				_g.push(tmp);
			}
		}
		var insetDist = _g;
		return com_watabou_mfcg_model_wards_Ward.inset(this.shape, insetDist, corners);
	}
	, createAlleys: function (p) {
		var params = this.district.alleys;
		var bisector = new com_watabou_mfcg_utils_Bisector(p, params.minSq * params.blockSize, this.district.alleys.gridChaos * 16);
		bisector.getGap = function (cut) {
			return 1.2;
		};
		bisector.processCut = $bind(this, this.semiSmooth);
		if (!this.urban) {
			bisector.getMinArea = $bind(this, this.getBlockSize);
		}
		var _g = 0;
		var _g1 = bisector.partition();
		while (_g < _g1.length) {
			var face = _g1[_g];
			++_g;
			var area = com_watabou_geom_polygons_PolyCore.area(face);
			var params = this.district.alleys;
			var randomArea = params.minSq * Math.pow(2, params.sizeChaos * (2 * ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) - 1));
			if (area < randomArea) {
				this.createBlock(face, true);
			} else if (this.church == null && area <= randomArea * 4) {
				this.createChurch(face);
			} else {
				this.createBlock(face);
			}
		}
	}
	, semiSmooth: function (poly) {
		var a = poly[0];
		var b = poly[1];
		var c = poly[2];
		var d3 = openfl_geom_Point.distance(a, c);
		var triArea = Math.abs(com_watabou_geom_polygons_PolyCore.area(poly));
		if (triArea / d3 < 1 || triArea / (d3 * d3) < 0.01) {
			return [a, c];
		}
		var d1 = openfl_geom_Point.distance(a, b);
		var d2 = openfl_geom_Point.distance(b, c);
		var p1 = b.subtract(a);
		var p2 = c.subtract(b);
		var cos = (p1.x * p2.x + p1.y * p2.y) / d1 / d2;
		var chance = Math.pow((1 + cos) / 2, 1);
		if (chance == null) {
			chance = 0.5;
		}
		if (!((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance)) {
			return poly;
		}
		var curve;
		var minFront = this.district.alleys.minFront;
		if (d1 < d2) {
			var f = Math.log(d1 / minFront) / Math.log(2);
			var chance = f - (f | 0);
			if (chance == null) {
				chance = 0.5;
			}
			var n = (f | 0) + ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance ? 1 : 0);
			var b1 = c.subtract(b);
			var t = d1 / d2;
			var c1 = new openfl_geom_Point(b.x + b1.x * t, b.y + b1.y * t);
			curve = com_watabou_geom_Chaikin.render([a, b, c1], false, n);
			curve.push(c);
		} else {
			var f = Math.log(d2 / minFront) / Math.log(2);
			var chance = f - (f | 0);
			if (chance == null) {
				chance = 0.5;
			}
			var n = (f | 0) + ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance ? 1 : 0);
			var b1 = a.subtract(b);
			var t = d2 / d1;
			var a1 = new openfl_geom_Point(b.x + b1.x * t, b.y + b1.y * t);
			curve = com_watabou_geom_Chaikin.render([a1, b, c], false, n);
			curve.unshift(a);
		}
		return curve;
	}
	, createBlock: function (shape, single) {
		if (single == null) {
			single = false;
		}
		var block = new com_watabou_mfcg_model_blocks_Block(this, shape, single);
		if (block.lots.length > 0) {
			this.blocks.push(block);
		}
	}
	, createChurch: function (shape) {
		var obb = com_watabou_geom_polygons_PolyBounds.obb(shape);
		var v1 = obb[0].subtract(obb[1]);
		var v2 = obb[2].subtract(obb[1]);
		var long = v1.get_length() > v2.get_length() ? v1 : v2;
		var minOffset = this.district.alleys.minFront;
		var ofs2axis = minOffset / long.get_length();
		var ratio = ofs2axis > 0.5 ? 0.5 : ofs2axis + (1 - 2 * ofs2axis) * (((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 3);
		var a = obb[1];
		var p1 = new openfl_geom_Point(a.x + long.x * ratio, a.y + long.y * ratio);
		var p2 = p1.add(new openfl_geom_Point(-long.y, long.x));
		var halves = com_watabou_geom_polygons_PolyCut.cut(shape, p1, p2);
		var large = com_watabou_utils_ArrayExtender.max(halves, function (h) {
			return com_watabou_geom_polygons_PolyCore.compactness(h);
		});
		this.church = new com_watabou_mfcg_model_blocks_Block(this, large, true);
		this.blocks.push(this.church);
	}
	, filter: function () {
		var _gthis = this;
		var vertices = new haxe_ds_ObjectMap();
		var a = this.border;
		var prev = a[a.length - 1];
		var _g = 0;
		var _g1 = this.border;
		while (_g < _g1.length) {
			var edge = _g1[_g];
			++_g;
			var v = edge.origin;
			var k = v.point;
			var v1;
			if (this.inner.indexOf(v) != -1) {
				v1 = 1.0;
			} else {
				var _g2 = [];
				var e = prev;
				var v2;
				if (e.data == null) {
					v2 = 0.0;
				} else {
					var _g3 = e.data;
					switch (_g3._hx_index) {
						case 2:
							v2 = 0.3;
							break;
						case 3:
							v2 = 0.5;
							break;
						case 4:
							var _g4 = _g3.c;
							v2 = 0.1;
							break;
						default:
							v2 = 0.0;
					}
				}
				_g2.push(v2);
				var e1 = edge;
				var v3;
				if (e1.data == null) {
					v3 = 0.0;
				} else {
					var _g5 = e1.data;
					switch (_g5._hx_index) {
						case 2:
							v3 = 0.3;
							break;
						case 3:
							v3 = 0.5;
							break;
						case 4:
							var _g6 = _g5.c;
							v3 = 0.1;
							break;
						default:
							v3 = 0.0;
					}
				}
				_g2.push(v3);
				var edges = _g2;
				v1 = Math.max(edges[0], edges[1]);
			}
			vertices.set(k, v1);
			prev = edge;
		}
		var n = Math.sqrt(this.patches.length);
		var z = 0.5 * n - 0.5;
		var _g = 0;
		var _g1 = this.blocks;
		while (_g < _g1.length) {
			var block = _g1[_g];
			++_g;
			var _g2 = [];
			var _g3 = 0;
			var _g4 = block.lots;
			while (_g3 < _g4.length) {
				var v = _g4[_g3];
				++_g3;
				var c = com_watabou_geom_polygons_PolyCore.center(v);
				var v1 = _gthis.interpolate(c, vertices);
				var tmp;
				if (!isNaN(v1)) {
					var chance = v1 * n - z;
					if (chance == null) {
						chance = 0.5;
					}
					tmp = (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance;
				} else {
					tmp = false;
				}
				if (tmp) {
					_g2.push(v);
				}
			}
			block.lots = _g2;
		}
		var _g = [];
		var _g1 = 0;
		var _g2 = this.blocks;
		while (_g1 < _g2.length) {
			var v = _g2[_g1];
			++_g1;
			if (v.lots.length > 0) {
				_g.push(v);
			}
		}
		var nonEmpty = _g;
		var a = this.blocks;
		a.splice(0, a.length);
		var a = this.blocks;
		var _g = 0;
		while (_g < nonEmpty.length) {
			var e = nonEmpty[_g];
			++_g;
			a.push(e);
		}
	}
	, getTris: function () {
		if (this.tris == null) {
			var _g = [];
			var _g1 = 0;
			var _g2 = com_watabou_geom_Triangulation.earcut(this.shape);
			while (_g1 < _g2.length) {
				var tri = _g2[_g1];
				++_g1;
				_g.push([this.shape[tri[0]], this.shape[tri[1]], this.shape[tri[2]]]);
			}
			this.tris = _g;
		}
		return this.tris;
	}
	, interpolate: function (p, vertices) {
		var _g = 0;
		var _g1 = this.getTris();
		while (_g < _g1.length) {
			var tri = _g1[_g];
			++_g;
			var b = com_watabou_geom_GeomUtils.barycentric(tri[0], tri[1], tri[2], p);
			if (b.x >= 0 && b.y >= 0 && b.z >= 0) {
				return b.x * vertices.h[tri[0].__id__] + b.y * vertices.h[tri[1].__id__] + b.z * vertices.h[tri[2].__id__];
			}
		}
		return NaN;
	}
	, isInnerVertex: function (v) {
		var _g = 0;
		var _g1 = v.edges;
		while (_g < _g1.length) {
			var e = _g1[_g];
			++_g;
			var patch = e.face.data;
			if (!patch.withinCity && !patch.waterbody) {
				return false;
			}
		}
		return true;
	}
	, getBlockSize: function (poly) {
		var c = com_watabou_geom_polygons_PolyCore.center(poly);
		var params = this.district.alleys;
		return params.minSq * params.blockSize * this.interpolate(c, this.blockM);
	}
	, __class__: com_watabou_mfcg_model_wards_WardGroup
};
var com_watabou_mfcg_scenes_TestScene = function () {
	com_watabou_coogee_Scene.call(this);
	this.view = new openfl_display_Sprite();
	this.addChild(this.view);
	this.keyEvent.add($bind(this, this.onKey));
	this.reset1();
};
$hxClasses["com.watabou.mfcg.scenes.TestScene"] = com_watabou_mfcg_scenes_TestScene;
com_watabou_mfcg_scenes_TestScene.__name__ = "com.watabou.mfcg.scenes.TestScene";
com_watabou_mfcg_scenes_TestScene.__super__ = com_watabou_coogee_Scene;
com_watabou_mfcg_scenes_TestScene.prototype = $extend(com_watabou_coogee_Scene.prototype, {
	layout: function () {
		this.view.set_x(this.rWidth / 2);
		this.view.set_y(this.rHeight / 2);
	}
	, onKey: function (key, down) {
		if (down) {
			if (key == 13) {
				this.reset();
			}
		}
	}
	, reset1: function () {
		this.view.set_scaleX(this.view.set_scaleY(40));
		var g = this.view.get_graphics();
		g.clear();
		var poly = [{ x: 16.0942418273704, y: 25.0948346360535 }, { x: 11.6933785204568, y: 22.7933411733268 }, { x: 8.78833166721554, y: 26.8213894334133 }, { x: 6.96717811309231, y: 29.3465447151093 }, { x: 4.48870913973456, y: 27.5590615403783 }, { x: 6.07083955668835, y: 25.3019213668817 }, { x: 7.87421244421238, y: 21.66710320056 }, { x: 8.04118594975039, y: 21.227913643856 }, { x: 16.1898380823096, y: 24.9120376577203 }];
		var _g = [];
		var _g1 = 0;
		while (_g1 < poly.length) {
			var p = poly[_g1];
			++_g1;
			_g.push(new openfl_geom_Point(p.x, p.y));
		}
		var poly = _g;
		var aabb = com_watabou_geom_polygons_PolyBounds.rect(poly);
		var cx = aabb.x + aabb.width / 2;
		var cy = aabb.y + aabb.height / 2;
		com_watabou_geom_polygons_PolyTransform.asTranslate(poly, -cx, -cy);
		g.lineStyle(0.02, 0);
		com_watabou_utils_GraphicsExtender.drawPolygon(g, poly);
	}
	, reset: function () {
		var g = this.view.get_graphics();
		g.clear();
		var n = Math.floor(4 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 * 12);
		var _g = [];
		var _g1 = 0;
		var _g2 = n;
		while (_g1 < _g2) {
			var i = _g1++;
			_g.push(openfl_geom_Point.polar(360 * (1 - Math.abs(((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 2 - 1)), Math.PI * 2 * (i + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / n));
		}
		this.poly = _g;
		var d1 = 20;
		var d2 = 100;
		var _g = [];
		var _g1 = 0;
		var _g2 = n;
		while (_g1 < _g2) {
			var i = _g1++;
			_g.push((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < 0.5 ? d1 : d2);
		}
		this.dist = _g;
		haxe_Log.trace(this.dist, { fileName: "Source/com/watabou/mfcg/scenes/TestScene.hx", lineNumber: 83, className: "com.watabou.mfcg.scenes.TestScene", methodName: "reset" });
		this.result = com_watabou_mfcg_utils_PolyUtils.simpleInset(this.poly, this.dist);
		g.lineStyle(2, 0);
		com_watabou_utils_GraphicsExtender.drawPolygon(g, this.poly);
		g.lineStyle(1, 16711680);
		com_watabou_utils_GraphicsExtender.drawPolygon(g, this.result);
		g.lineStyle(5, 255, 0.3);
		com_watabou_utils_GraphicsExtender.drawPolygon(g, com_watabou_mfcg_utils_PolyUtils.inset(this.poly, this.dist));
	}
	, __class__: com_watabou_mfcg_scenes_TestScene
});
var com_watabou_mfcg_scenes_TownScene = function () {
	com_watabou_coogee_Scene.call(this);
	com_watabou_mfcg_scenes_TownScene.instance = this;
	this.model = com_watabou_mfcg_model_City.instance;
	this.createOverlays();
	this.updateOverlays();
	this.toggleOverlays();
	if (!com_watabou_mfcg_Main.preview) {
		this.keyEvent.add($bind(this, this.onKeyEvent));
	}
	com_watabou_mfcg_scenes_TownScene.context = new com_watabou_mfcg_scenes_ContextMenu();
};
$hxClasses["com.watabou.mfcg.scenes.TownScene"] = com_watabou_mfcg_scenes_TownScene;
com_watabou_mfcg_scenes_TownScene.__name__ = "com.watabou.mfcg.scenes.TownScene";
com_watabou_mfcg_scenes_TownScene.applyPalette = function (pal) {
	com_watabou_mfcg_mapping_Style.setPalette(pal, true);
	com_watabou_mfcg_model_District.updateColors(com_watabou_mfcg_model_City.instance.districts);
	com_watabou_coogee_Game.switchScene(com_watabou_mfcg_scenes_ViewScene);
};
com_watabou_mfcg_scenes_TownScene.editColors = function () {
	if (com_watabou_coogee_ui_UI.findWidnow(com_watabou_coogee_ui_forms_PaletteForm) != null) {
		return;
	}
	var form = new com_watabou_coogee_ui_forms_PaletteForm(com_watabou_mfcg_scenes_TownScene.applyPalette, ["Default", "default", "Ink", "ink", "Black & White", "bw", "Vivid", "vivid", "Natural", "natural", "Modern", "modern"]);
	form.getName = com_watabou_coogee_ui_forms_PaletteForm.swatches(["colorPaper", "colorRoof", "colorDark"]);
	com_watabou_mfcg_mapping_Style.fillForm(form);
	com_watabou_coogee_ui_UI.showDialog(form, "Color scheme");
};
com_watabou_mfcg_scenes_TownScene.updateMap = function () {
	com_watabou_mfcg_scenes_TownScene.instance.drawMap();
};
com_watabou_mfcg_scenes_TownScene.reloadScene = function () {
	com_watabou_coogee_Game.switchScene(com_watabou_mfcg_scenes_ViewScene);
};
com_watabou_mfcg_scenes_TownScene.__super__ = com_watabou_coogee_Scene;
com_watabou_mfcg_scenes_TownScene.prototype = $extend(com_watabou_coogee_Scene.prototype, {
	get_mapScale: function () {
		var scaleX = this.rWidth / (this.model.maxx - this.model.minx + 40);
		var scaleY = this.rHeight / (this.model.maxy - this.model.miny + 40);
		var scMin = Math.min(scaleX, scaleY);
		var scMax = Math.max(scaleX, scaleY);
		if (scMax / scMin > 2) {
			return scMax / 2;
		} else {
			return scMin;
		}
	}
	, activate: function () {
		com_watabou_coogee_Scene.prototype.activate.call(this);
		this.stage.set_color(this.getBgColor());
		if (!com_watabou_mfcg_Main.preview) {
			this.addEventListener("rightClick", $bind(this, this.onRightClick));
		}
	}
	, deactivate: function () {
		com_watabou_coogee_Scene.prototype.deactivate.call(this);
		this.removeEventListener("rightClick", $bind(this, this.onRightClick));
	}
	, onRightClick: function (e) {
		this.onContext(com_watabou_mfcg_scenes_TownScene.context);
		com_watabou_mfcg_scenes_TownScene.context.show();
	}
	, onEsc: function () {
		if (!com_watabou_coogee_ui_UI.hideMenu()) {
			com_watabou_coogee_Scene.prototype.onEsc.call(this);
		}
	}
	, onContext: function (context) {
	}
	, onMapContext: function (context) {
	}
	, onKeyEvent: function (keyCode, down) {
	}
	, createOverlays: function () {
		this.overlays = [];
		this.overlays.push(this.pins = new com_watabou_mfcg_scenes_overlays_PinsOverlay(this));
		this.addChild(this.pins);
		this.overlays.push(this.markers = new com_watabou_mfcg_scenes_overlays_MarkersOverlay(this));
		this.addChild(this.markers);
		this.overlays.push(this.scBar = new com_watabou_mfcg_scenes_overlays_ScaleBarOverlay(this));
		this.addChild(this.scBar);
		this.overlays.push(this.compass = new com_watabou_mfcg_scenes_overlays_CompassOverlay(this));
		this.addChild(this.compass);
		this.overlays.push(this.legend = new com_watabou_mfcg_scenes_overlays_LegendOverlay(this));
		this.addChild(this.legend);
	}
	, updateOverlays: function () {
		var _g = 0;
		var _g1 = this.overlays;
		while (_g < _g1.length) {
			var o = _g1[_g];
			++_g;
			o.update(this.model);
		}
	}
	, resetOverlays: function () {
		var _g = 0;
		var _g1 = this.overlays;
		while (_g < _g1.length) {
			var o = _g1[_g];
			++_g;
			this.removeChild(o);
		}
		this.createOverlays();
		this.updateOverlays();
		this.toggleOverlays();
		var _g = 0;
		var _g1 = this.overlays;
		while (_g < _g1.length) {
			var o = _g1[_g];
			++_g;
			o.setSize(this.rWidth, this.rHeight);
		}
	}
	, layout: function () {
		this.layoutMap();
		this.layoutLabels();
		var _g = 0;
		var _g1 = this.overlays;
		while (_g < _g1.length) {
			var o = _g1[_g];
			++_g;
			o.setSize(this.rWidth, this.rHeight);
		}
	}
	, recreateMap: function () {
		var _gthis = this;
		if (this.map != null) {
			this.removeChild(this.map);
		}
		this.map = new com_watabou_mfcg_mapping_FormalMap(this.model);
		this.map.addEventListener("rightClick", function (e) {
			_gthis.onMapContext(com_watabou_mfcg_scenes_TownScene.context);
		});
		this.addChildAt(this.map, 0);
	}
	, layoutMap: function () {
		var scale = this.get_mapScale();
		var invScale = 1 / scale;
		if (this.map == null || com_watabou_mfcg_mapping_Style.lineInvScale != invScale) {
			com_watabou_mfcg_mapping_Style.lineInvScale = invScale;
			if (this.map == null || true) {
				this.recreateMap();
			}
		}
		this.map.set_scaleX(this.map.set_scaleY(scale));
		this.map.set_x(this.rWidth / 2 - (this.model.maxx + this.model.minx) / 2 * scale);
		this.map.set_y(this.rHeight / 2 - (this.model.maxy + this.model.miny) / 2 * scale);
		var tl = this.map.globalToLocal(this.localToGlobal(new openfl_geom_Point(0, 0)));
		var br = this.map.globalToLocal(this.localToGlobal(new openfl_geom_Point(this.rWidth, this.rHeight)));
		this.map.updateBounds(tl.x, br.x, tl.y, br.y);
	}
	, layoutLabels: function () {
		this.map.layoutLabels();
		this.toggleOverlays();
	}
	, drawMap: function () {
		this.stage.set_color(this.getBgColor());
		this.recreateMap();
		this.layoutMap();
		this.layoutLabels();
	}
	, getBgColor: function () {
		if (this.model.waterEdge.length > 0) {
			return com_watabou_mfcg_mapping_Style.colorWater;
		} else {
			return com_watabou_mfcg_mapping_Style.colorPaper;
		}
	}
	, loadPreset: function (id) {
		com_watabou_mfcg_scenes_TownScene.applyPalette(com_watabou_utils_Palette.fromJSON(openfl_utils_Assets.getText(id)));
	}
	, toggleOverlays: function (modified) {
		if (modified == null) {
			modified = false;
		}
		var legendNeeded = com_watabou_system_State.get("districts", "Curved") == "Legend" || com_watabou_system_State.get("landmarks") == "Legend";
		this.legend.set_visible(legendNeeded);
		if (this.legend.get_visible()) {
			this.legend.update(this.model);
		}
		this.pins.set_visible(com_watabou_system_State.get("districts", "Curved") == "Legend");
		this.markers.set_visible(com_watabou_system_State.get("landmarks") != "Hidden");
		if (this.map != null) {
			var districts = com_watabou_system_State.get("districts", "Curved");
			this.map.showLabels(districts == "Straight" || districts == "Curved");
		}
		this.scBar.set_visible(com_watabou_system_State.get("scale_bar", true) && !legendNeeded);
		this.compass.set_visible(com_watabou_system_State.get("compass", true));
	}
	, arrangeOverlays: function () {
		if (this.legend.get_visible() && this.legend.get_position() == com_watabou_mfcg_scenes_overlays_Position.BOTTOM_RIGHT) {
			this.compass.set_position(com_watabou_mfcg_scenes_overlays_Position.BOTTOM_LEFT);
		} else {
			this.compass.set_position(com_watabou_mfcg_scenes_overlays_Position.BOTTOM_RIGHT);
		}
	}
	, getOverlay: function (cl) {
		var _g = 0;
		var _g1 = this.overlays;
		while (_g < _g1.length) {
			var o = _g1[_g];
			++_g;
			if (js_Boot.__instanceof(o, cl)) {
				return o;
			}
		}
		return null;
	}
	, __class__: com_watabou_mfcg_scenes_TownScene
	, __properties__: $extend(com_watabou_coogee_Scene.prototype.__properties__, { get_mapScale: "get_mapScale" })
});
var com_watabou_mfcg_scenes_ContextMenu = function () {
	this.wipe();
};
$hxClasses["com.watabou.mfcg.scenes.ContextMenu"] = com_watabou_mfcg_scenes_ContextMenu;
com_watabou_mfcg_scenes_ContextMenu.__name__ = "com.watabou.mfcg.scenes.ContextMenu";
com_watabou_mfcg_scenes_ContextMenu.prototype = {
	wipe: function () {
		this.items = [];
	}
	, group: function () {
		if (this.items.length > 0 && this.items[this.items.length - 1] != null) {
			this.items.push(null);
		}
	}
	, show: function (parent) {
		var menu = new com_watabou_coogee_ui_Menu();
		var _g = 0;
		var _g1 = this.items;
		while (_g < _g1.length) {
			var item = _g1[_g];
			++_g;
			if (item == null) {
				menu.addSeparator();
			} else {
				menu.add(item);
			}
		}
		this.wipe();
		if (parent != null) {
			com_watabou_coogee_ui_UI.showMenuAt(menu, parent.get_x() + parent.rWidth, parent.get_y() + parent.rHeight + 2);
		} else {
			com_watabou_coogee_ui_UI.showMenu(menu);
		}
	}
	, addItem: function (label, callback, checked) {
		if (checked == null) {
			checked = false;
		}
		var item = new com_watabou_coogee_ui_MenuItem(label, null, callback);
		item.setCheck(checked);
		this.items.push(item);
	}
	, addSubmenu: function (label, submenu) {
		var item = new com_watabou_coogee_ui_MenuItem(label, submenu);
		this.items.push(item);
	}
	, addSeparator: function () {
		this.items.push(null);
	}
	, isEmpty: function () {
		return this.items.length == 0;
	}
	, __class__: com_watabou_mfcg_scenes_ContextMenu
};
var com_watabou_mfcg_ui_forms_ToolForm = function () {
	com_watabou_coogee_ui_Form.call(this);
};
$hxClasses["com.watabou.mfcg.ui.forms.ToolForm"] = com_watabou_mfcg_ui_forms_ToolForm;
com_watabou_mfcg_ui_forms_ToolForm.__name__ = "com.watabou.mfcg.ui.forms.ToolForm";
com_watabou_mfcg_ui_forms_ToolForm.loadSaved = function (classes) {
	com_watabou_mfcg_ui_forms_ToolForm.saved = com_watabou_system_State.get("tools");
	if (com_watabou_mfcg_ui_forms_ToolForm.saved == null) {
		var this1 = {};
		com_watabou_mfcg_ui_forms_ToolForm.saved = this1;
	}
	if (classes != null) {
		var _g = 0;
		while (_g < classes.length) {
			var cl = classes[_g];
			++_g;
			var key = cl.__name__;
			var saved = com_watabou_mfcg_ui_forms_ToolForm.saved[key];
			if (saved != null && saved.visible && com_watabou_coogee_ui_UI.findWidnow(cl) == null) {
				var form = Type.createInstance(cl, []);
				com_watabou_coogee_ui_UI.showDialog(form);
				form.restore();
			}
		}
	}
};
com_watabou_mfcg_ui_forms_ToolForm.__super__ = com_watabou_coogee_ui_Form;
com_watabou_mfcg_ui_forms_ToolForm.prototype = $extend(com_watabou_coogee_ui_Form.prototype, {
	onShow: function () {
		var _gthis = this;
		this.dialog.onMove.add($bind(this, this.onMove));
		this.dialog.onMinimize.add(function (wnd) {
			_gthis.save(true);
		});
		this.dialog.minimizable = true;
	}
	, onHide: function () {
		this.save(false);
	}
	, onMove: function (wnd) {
		var pos = wnd.getAdjustment();
		if (pos != null) {
			wnd.set_x(pos.x);
			wnd.set_y(pos.y);
		}
		this.save(true);
	}
	, onKey: function (key) {
		return false;
	}
	, id: function () {
		var c = js_Boot.getClass(this);
		return c.__name__;
	}
	, save: function (visible) {
		var _this = this.dialog;
		var c = new openfl_geom_Point(_this.get_x() + _this.rWidth / 2, _this.get_y() + _this.rHeight / 2);
		com_watabou_mfcg_ui_forms_ToolForm.saved[this.id()] = { x: c.x / com_watabou_coogee_ui_UI.layer.get_width(), y: c.y / com_watabou_coogee_ui_UI.layer.get_height(), visible: visible, minimized: this.dialog.minimized };
		com_watabou_system_State.set("tools", com_watabou_mfcg_ui_forms_ToolForm.saved);
	}
	, restore: function () {
		var id = this.id();
		var pos = com_watabou_mfcg_ui_forms_ToolForm.saved[id];
		if (pos != null) {
			this.dialog.set_x(pos.x * com_watabou_coogee_ui_UI.layer.get_width() - this.dialog.get_width() / 2 | 0);
			this.dialog.set_y(pos.y * com_watabou_coogee_ui_UI.layer.get_height() - this.dialog.get_height() / 2 | 0);
			this.dialog.setMinimized(pos.minimized);
			var pos = this.dialog.getAdjustment();
			if (pos != null) {
				this.dialog.set_x(pos.x);
				this.dialog.set_y(pos.y);
			}
		}
		this.save(true);
	}
	, forceDisplay: function () {
		com_watabou_mfcg_ui_forms_ToolForm.saved[this.id()].visible = true;
	}
	, __class__: com_watabou_mfcg_ui_forms_ToolForm
});
var com_watabou_mfcg_ui_forms_GenerateForm = function () {
	var _gthis = this;
	com_watabou_mfcg_ui_forms_ToolForm.call(this);
	var params = this.createFeaturesTab();
	var roads = this.createRoadsTab();
	this.tabs = new com_watabou_coogee_ui_layouts_Tabs();
	this.tabs.addTab("Features", params);
	this.tabs.addTab("Roads", roads);
	this.buttons = new com_watabou_coogee_ui_layouts_VBox();
	this.buttons.setMargins(10, 8);
	var btnSmall = new com_watabou_coogee_ui_Button("Small");
	btnSmall.set_width(80);
	btnSmall.click.add(function () {
		_gthis.build(Math.floor(10 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 * 10));
	});
	this.buttons.add(btnSmall);
	var btnMedium = new com_watabou_coogee_ui_Button("Medium");
	btnMedium.set_width(80);
	btnMedium.click.add(function () {
		_gthis.build(Math.floor(20 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 * 20));
	});
	this.buttons.add(btnMedium);
	var btnLarge = new com_watabou_coogee_ui_Button("Large");
	btnLarge.set_width(80);
	btnLarge.click.add(function () {
		_gthis.build(Math.floor(40 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 * 40));
	});
	this.buttons.add(btnLarge);
	this.txtSize = new com_watabou_coogee_ui_TextInput();
	this.txtSize.set_centered(true);
	this.txtSize.set_width(80);
	this.txtSize.set_prompt("Size");
	this.txtSize.set_restrict("0-9");
	this.txtSize.enter.add($bind(this, this.onCustomSize));
	this.buttons.add(this.txtSize);
	var btnRebuild = new com_watabou_coogee_ui_Button("Rebuild");
	btnRebuild.set_width(80);
	btnRebuild.click.add($bind(this, this.rebuild));
	this.buttons.add(btnRebuild);
	this.bg = com_watabou_coogee_ui_SolidRect.light();
	this.add(this.tabs);
	this.add(this.bg);
	this.add(this.buttons);
};
$hxClasses["com.watabou.mfcg.ui.forms.GenerateForm"] = com_watabou_mfcg_ui_forms_GenerateForm;
com_watabou_mfcg_ui_forms_GenerateForm.__name__ = "com.watabou.mfcg.ui.forms.GenerateForm";
com_watabou_mfcg_ui_forms_GenerateForm.__super__ = com_watabou_mfcg_ui_forms_ToolForm;
com_watabou_mfcg_ui_forms_GenerateForm.prototype = $extend(com_watabou_mfcg_ui_forms_ToolForm.prototype, {
	getTitle: function () {
		return "Generate";
	}
	, layout: function () {
		this.buttons.set_x(this.bg.set_x(this.tabs.get_width()));
		this.rWidth = this.tabs.get_width() + this.buttons.get_width();
		this.rHeight = Math.max(this.tabs.get_height(), this.buttons.get_height());
		this.bg.setSize(this.buttons.get_width(), this.rHeight);
	}
	, onKey: function (key) {
		if (key == 13) {
			this.build(com_watabou_mfcg_model_City.nextSize);
		} else {
			return false;
		}
		return true;
	}
	, getCheckBox: function (id, label) {
		var chk = new com_watabou_coogee_ui_CheckBox(label);
		chk.set_value(com_watabou_system_State.get(id));
		return chk;
	}
	, createFeaturesTab: function () {
		var _gthis = this;
		var tab = new com_watabou_coogee_ui_layouts_VBox();
		this.chkRandom = this.getCheckBox("random", "Random");
		this.chkRandom.changed.add($bind(this, this.onRandom));
		tab.add(this.chkRandom);
		var sep = com_watabou_coogee_ui_SolidRect.black();
		sep.set_height(2);
		sep.halign = "fill";
		tab.add(sep);
		var grid = new com_watabou_coogee_ui_layouts_Grid(2);
		grid.setMargins(0, 8);
		var f = 0;
		this.chkFeatures = new haxe_ds_StringMap();
		while (f < com_watabou_mfcg_ui_forms_GenerateForm.features.length) {
			var label = com_watabou_mfcg_ui_forms_GenerateForm.features[f];
			var key = [com_watabou_mfcg_ui_forms_GenerateForm.features[f + 1]];
			var chk = this.getCheckBox(key[0], label);
			chk.set_enabled(!this.chkRandom.get_value() || com_watabou_mfcg_ui_forms_GenerateForm.nonRandom.indexOf(label) != -1);
			chk.changed.add((function (key) {
				return function (value) {
					if (key[0] == "citadel" && !value) {
						_gthis.chkFeatures.h["urban_castle"].set_value(false);
					}
					if (key[0] == "urban_castle" && value) {
						_gthis.chkFeatures.h["citadel"].set_value(true);
					}
				};
			})(key));
			this.chkFeatures.h[key[0]] = chk;
			grid.add(chk);
			f += 2;
		}
		tab.add(grid);
		var sep = com_watabou_coogee_ui_SolidRect.black();
		sep.set_height(2);
		sep.halign = "fill";
		return tab;
	}
	, onRandom: function (value) {
		var h = this.chkFeatures.h;
		var chk_h = h;
		var chk_keys = Object.keys(h);
		var chk_length = chk_keys.length;
		var chk_current = 0;
		while (chk_current < chk_length) {
			var chk = chk_h[chk_keys[chk_current++]];
			if (com_watabou_mfcg_ui_forms_GenerateForm.nonRandom.indexOf(chk.get_text()) == -1) {
				chk.set_enabled(!value);
			}
		}
	}
	, createRoadsTab: function () {
		var _gthis = this;
		var tab = new com_watabou_coogee_ui_layouts_Grid(2);
		tab.setMargins(10, 8);
		this.chkDefault = new com_watabou_coogee_ui_CheckBox();
		tab.add(this.chkDefault);
		tab.add(new com_watabou_coogee_ui_Label("Default number"));
		this.chkMaximum = this.getCheckBox("hub", null);
		tab.add(this.chkMaximum);
		tab.add(new com_watabou_coogee_ui_Label("Maximum number"));
		this.chkSpecific = new com_watabou_coogee_ui_CheckBox();
		this.chkSpecific.valign = "center";
		tab.add(this.chkSpecific);
		var nGates = com_watabou_system_State.get("gates");
		this.txtGates = new com_watabou_coogee_ui_TextInput(nGates > -1 ? nGates == null ? "null" : "" + nGates : "");
		this.txtGates.set_restrict("0-9");
		this.txtGates.set_prompt("Number");
		this.txtGates.set_width(100);
		this.txtGates.update.add(function (text) {
			_gthis.chkDefault.set_value(false);
			_gthis.chkMaximum.set_value(false);
			_gthis.chkSpecific.set_value(true);
		});
		this.txtGates.enter.add(function (text) {
			_gthis.build(com_watabou_mfcg_model_City.nextSize);
		});
		tab.add(this.txtGates);
		var index = nGates > -1 ? 2 : com_watabou_system_State.get("hub") ? 1 : 0;
		new com_watabou_coogee_ui_RadioGroup([this.chkDefault, this.chkMaximum, this.chkSpecific], index);
		return tab;
	}
	, onCustomSize: function (txt) {
		var n = Std.parseInt(txt);
		if (n != null && n >= 5 && n <= 200) {
			this.build(n);
		}
	}
	, saveFeatures: function () {
		com_watabou_system_State.set("random", this.chkRandom.get_value());
		var h = this.chkFeatures.h;
		var _g_h = h;
		var _g_keys = Object.keys(h);
		var _g_length = _g_keys.length;
		var _g_current = 0;
		while (_g_current < _g_length) {
			var key = _g_keys[_g_current++];
			var _g1_key = key;
			var _g1_value = _g_h[key];
			var id = _g1_key;
			var chk = _g1_value;
			com_watabou_system_State.set(id, chk.get_value());
		}
		com_watabou_system_State.set("hub", this.chkMaximum.get_value());
		com_watabou_system_State.set("gates", this.chkSpecific.get_value() ? Std.parseInt(this.txtGates.get_text()) : -1);
	}
	, build: function (size) {
		this.saveFeatures();
		new com_watabou_mfcg_model_City(com_watabou_mfcg_model_Blueprint.create(size, com_watabou_utils_Random.seed));
		com_watabou_coogee_Game.switchScene(com_watabou_mfcg_scenes_ViewScene);
		this.stage.set_focus(this);
	}
	, rebuild: function () {
		this.saveFeatures();
		new com_watabou_mfcg_model_City(com_watabou_mfcg_model_Blueprint.similar(com_watabou_mfcg_model_City.instance.bp));
		com_watabou_coogee_Game.switchScene(com_watabou_mfcg_scenes_ViewScene);
	}
	, update: function () {
		this.chkRandom.set_value(com_watabou_system_State.get("random", true));
		var h = this.chkFeatures.h;
		var _g_h = h;
		var _g_keys = Object.keys(h);
		var _g_length = _g_keys.length;
		var _g_current = 0;
		while (_g_current < _g_length) {
			var key = _g_keys[_g_current++];
			var _g1_key = key;
			var _g1_value = _g_h[key];
			var id = _g1_key;
			var chk = _g1_value;
			chk.set_value(com_watabou_system_State.get(id, true));
		}
		this.onRandom(this.chkRandom.get_value());
		var gates = com_watabou_system_State.get("gates");
		this.chkDefault.set_value(!com_watabou_system_State.get("hub") && gates <= -1);
		this.chkMaximum.set_value(com_watabou_system_State.get("hub") && gates <= -1);
		if (this.chkSpecific.set_value(gates > -1)) {
			this.txtGates.set_text(gates == null ? "null" : "" + gates);
		}
	}
	, fromURL: function () {
		if (com_watabou_coogee_ui_UI.findWidnow(com_watabou_mfcg_ui_forms_URLForm) == null) {
			com_watabou_coogee_ui_UI.showDialog(new com_watabou_mfcg_ui_forms_URLForm());
		}
	}
	, __class__: com_watabou_mfcg_ui_forms_GenerateForm
});
var com_watabou_mfcg_ui_forms_TownForm = function () {
	var _gthis = this;
	com_watabou_mfcg_ui_forms_ToolForm.call(this);
	this.setMargins(10, 8);
	this.model = com_watabou_mfcg_model_City.instance;
	this.txtName = new com_watabou_coogee_ui_TextInput(this.model.name);
	this.txtName.set_centered(true);
	this.txtName.enter.add($bind(this, this.onEnterName));
	this.txtName.leave.add(function () {
		_gthis.txtName.set_text(_gthis.model.name);
	});
	this.txtName.set_width(200);
	this.add(this.txtName);
	this.townInfo = new com_watabou_mfcg_ui_forms_TownInfo(this.model);
	this.townInfo.halign = "center";
	this.add(this.townInfo);
	this.addSection("Reroll names");
	this.addButtonRow("Town", $bind(this, this.rerollName), "Districts", ($_ = this.model, $bind($_, $_.rerollDistricts)));
	this.addSection("Points of interest");
	this.addButtonRow("Load", $bind(this, this.onLoadPOIs), "Clear", $bind(this, this.onClearPOIs));
	this.addSeparator();
	this.addButtonRow("Warp", $bind(this, this.onWarp), "Overworld", $bind(this, this.onOverworld));
	var row = new com_watabou_coogee_ui_layouts_HBox();
	row.setMargins(0, 8);
	var btnURL = new com_watabou_coogee_ui_Button("Copy URL");
	btnURL.set_width(96);
	btnURL.click.add($bind(this, this.onCopyURL));
	row.add(btnURL);
	var btnExport = new com_watabou_coogee_ui_MultiAction("Export", ["PNG", "SVG", "JSON"]);
	btnExport.set_width(96);
	btnExport.action.add($bind(this, this.onExport));
	row.add(btnExport);
	this.add(row);
	com_watabou_mfcg_model_ModelDispatcher.newModel.add($bind(this, this.onNewModel));
	com_watabou_mfcg_model_ModelDispatcher.titleChanged.add($bind(this, this.onTitleChanged));
	com_watabou_mfcg_model_ModelDispatcher.geometryChanged.add($bind(this, this.onGeometryChanged));
};
$hxClasses["com.watabou.mfcg.ui.forms.TownForm"] = com_watabou_mfcg_ui_forms_TownForm;
com_watabou_mfcg_ui_forms_TownForm.__name__ = "com.watabou.mfcg.ui.forms.TownForm";
com_watabou_mfcg_ui_forms_TownForm.__super__ = com_watabou_mfcg_ui_forms_ToolForm;
com_watabou_mfcg_ui_forms_TownForm.prototype = $extend(com_watabou_mfcg_ui_forms_ToolForm.prototype, {
	onHide: function () {
		com_watabou_mfcg_ui_forms_ToolForm.prototype.onHide.call(this);
		com_watabou_mfcg_model_ModelDispatcher.newModel.remove($bind(this, this.onNewModel));
		com_watabou_mfcg_model_ModelDispatcher.titleChanged.remove($bind(this, this.onTitleChanged));
		com_watabou_mfcg_model_ModelDispatcher.geometryChanged.remove($bind(this, this.onGeometryChanged));
	}
	, getTitle: function () {
		return "Settlement";
	}
	, addSeparator: function () {
		var sep = com_watabou_coogee_ui_SolidRect.black();
		sep.set_height(2);
		sep.halign = "fill";
		this.add(sep);
	}
	, addSection: function (name) {
		this.addSeparator();
		this.add(new com_watabou_coogee_ui_Label(name));
	}
	, addButtonRow: function (label1, on1, label2, on2) {
		var row = new com_watabou_coogee_ui_layouts_HBox();
		row.setMargins(0, 8);
		var w = 96.;
		var btn1 = new com_watabou_coogee_ui_Button(label1);
		btn1.set_width(w);
		btn1.click.add(on1);
		row.add(btn1);
		var btn2 = new com_watabou_coogee_ui_Button(label2);
		btn2.set_width(w);
		btn2.click.add(on2);
		row.add(btn2);
		this.add(row);
	}
	, onEnterName: function (name) {
		this.model.setName(name, true);
		this.stage.set_focus(this);
	}
	, rerollName: function () {
		this.model.setName(this.model.rerollName());
	}
	, onLoadPOIs: function () {
		var _gthis = this;
		var fr = new openfl_net_FileReference();
		fr.addEventListener("select", function (e) {
			fr.addEventListener("complete", $bind(_gthis, _gthis.onPOIsLoaded));
			fr.load();
		});
		fr.browse([new openfl_net_FileFilter("JSON list", "*.json")]);
	}
	, onPOIsLoaded: function (e) {
		var data = (js_Boot.__cast(e.target, openfl_net_FileReference)).data.toString();
		var list = JSON.parse(data);
		this.model.addLandmarks(list);
	}
	, onClearPOIs: function () {
		this.model.removeLandmarks();
	}
	, onWarp: function () {
		com_watabou_coogee_Game.switchScene(com_watabou_mfcg_scenes_WarpScene);
	}
	, onOverworld: function () {
		var request = new openfl_net_URLRequest("https://azgaar.github.io/Fantasy-Map-Generator/");
		request.method = "GET";
		request.data = this.model.getFMGParams();
		request.data.from = "MFCG";
		openfl_Lib.getURL(request, "fmg");
	}
	, onCopyURL: function () {
		com_watabou_mfcg_Buffer.write(com_watabou_system_URLState.getURL());
		com_watabou_coogee_ui_Toast.show("URL was copied to the clipboard");
	}
	, onExport: function (format) {
		switch (format) {
			case "JSON":
				com_watabou_mfcg_export_Export.asJSON();
				break;
			case "PNG":
				com_watabou_mfcg_export_Export.asPNG();
				break;
			case "SVG":
				com_watabou_mfcg_export_Export.asSVG();
				break;
		}
	}
	, onNewModel: function (model) {
		this.model = model;
		this.txtName.set_text(model.name);
		this.townInfo.update(model);
	}
	, onTitleChanged: function (title) {
		this.txtName.set_text(title);
	}
	, onGeometryChanged: function (patch) {
		this.townInfo.update(this.model);
	}
	, __class__: com_watabou_mfcg_ui_forms_TownForm
});
var com_watabou_mfcg_ui_forms_StyleForm = function () {
	com_watabou_mfcg_ui_forms_ToolForm.call(this);
	var tabs = new com_watabou_coogee_ui_layouts_Tabs();
	tabs.set_rowSize(4);
	tabs.addTab("Graphics", this.colors());
	tabs.addTab("Elements", this.elements());
	tabs.addTab("Buildings", this.buildings());
	tabs.addTab("Outline", this.outline());
	tabs.addTab("Text", this.text());
	tabs.addTab("Misc", this.misc());
	this.add(tabs);
	tabs.change.add(function (n) {
		com_watabou_mfcg_ui_forms_StyleForm.lastTab = n;
	});
	tabs.onTab(com_watabou_mfcg_ui_forms_StyleForm.lastTab);
};
$hxClasses["com.watabou.mfcg.ui.forms.StyleForm"] = com_watabou_mfcg_ui_forms_StyleForm;
com_watabou_mfcg_ui_forms_StyleForm.__name__ = "com.watabou.mfcg.ui.forms.StyleForm";
com_watabou_mfcg_ui_forms_StyleForm.__super__ = com_watabou_mfcg_ui_forms_ToolForm;
com_watabou_mfcg_ui_forms_StyleForm.prototype = $extend(com_watabou_mfcg_ui_forms_ToolForm.prototype, {
	getTitle: function () {
		return "Style";
	}
	, addCheckbox: function (label, id, def, parent, onUpdate) {
		var chk = new com_watabou_coogee_ui_CheckBox(label);
		chk.set_value(com_watabou_system_State.get(id, def));
		chk.changed.add(function (value) {
			com_watabou_system_State.set(id, value);
			onUpdate(value);
		});
		parent.add(chk);
		return chk;
	}
	, addDropDown: function (label, id, values, parent, onUpdate) {
		this.addLabel(label, parent);
		var cmb = com_watabou_coogee_ui_DropDown.ofStrings(values);
		cmb.set_value(com_watabou_system_State.get(id));
		cmb.set_width(120);
		cmb.set_centered(true);
		cmb.update.add(function (value) {
			com_watabou_system_State.set(id, value);
			onUpdate(value);
		});
		parent.add(cmb);
		return cmb;
	}
	, addFont: function (label, id, def, parent) {
		this.addLabel(label, parent);
		var textView = new com_watabou_coogee_ui_elements_TextView(com_watabou_system_State.get(id, def), com_watabou_coogee_ui_forms_FontForm.font2text, null, com_watabou_coogee_ui_elements_TextView.editInForm(com_watabou_coogee_ui_forms_FontForm, label, this));
		textView.set_width(180);
		textView.update.add(function (font) {
			com_watabou_system_State.set(id, font);
			com_watabou_coogee_Game.switchScene(com_watabou_mfcg_scenes_ViewScene);
		});
		parent.add(textView);
	}
	, addSeparator: function (parent) {
		var sep = com_watabou_coogee_ui_SolidRect.black();
		sep.set_height(2);
		sep.halign = "fill";
		parent.add(sep);
	}
	, addLabel: function (text, parent) {
		var label = new com_watabou_coogee_ui_Label(text);
		label.valign = "center";
		parent.add(label);
	}
	, colors: function () {
		var tab = new com_watabou_coogee_ui_layouts_VBox();
		tab.setMargins(10, 8);
		var btnColors = new com_watabou_coogee_ui_Button("Color scheme", com_watabou_mfcg_scenes_TownScene.editColors);
		tab.add(btnColors);
		this.addCheckbox("Thin lines", "thin_lines", false, tab, function (value) {
			com_watabou_mfcg_mapping_Style.thinLines = value;
			com_watabou_coogee_Game.switchScene(com_watabou_mfcg_scenes_ViewScene);
		});
		this.addCheckbox("Tint districts", "watercolours", false, tab, function (value) {
			com_watabou_mfcg_scenes_TownScene.updateMap();
		});
		this.addCheckbox("Weathered roofs", "weathered_roofs", false, tab, function (value) {
			if (com_watabou_system_State.get("display_mode", "Lots") != "Block") {
				com_watabou_mfcg_scenes_TownScene.updateMap();
			}
		});
		return tab;
	}
	, elements: function () {
		var _gthis = this;
		var tab = new com_watabou_coogee_ui_layouts_VBox();
		tab.setMargins(10, 8);
		var grid = new com_watabou_coogee_ui_layouts_Grid(2);
		grid.setMargins(0, 8);
		this.addLabel("Font size", grid);
		var sizes = ["Small", "Medium", "Large"];
		var cmbFont = com_watabou_coogee_ui_DropDown.ofInts(sizes);
		cmbFont.set_value(com_watabou_system_State.get("text_size", 1));
		cmbFont.set_width(120);
		cmbFont.set_centered(true);
		cmbFont.update.add(function (value) {
			com_watabou_system_State.set("text_size", value);
			com_watabou_coogee_Game.switchScene(com_watabou_mfcg_scenes_ViewScene);
		});
		grid.add(cmbFont);
		this.addDropDown("Districts", "districts", com_watabou_mfcg_Values.DISTRICTS_MODE, grid, function (value) {
			com_watabou_mfcg_scenes_TownScene.instance.toggleOverlays();
			var _g = 0;
			var _g1 = com_watabou_mfcg_model_City.instance.districts;
			while (_g < _g1.length) {
				var district = _g1[_g];
				++_g;
				district.updateGeometry();
			}
			com_watabou_mfcg_scenes_TownScene.instance.map.layoutLabels();
		});
		this.addDropDown("Landmarks", "landmarks", com_watabou_mfcg_Values.LANDMARK_MODES, grid, function (value) {
			com_watabou_mfcg_scenes_TownScene.instance.toggleOverlays();
		});
		tab.add(grid);
		this.addSeparator(tab);
		var grid = new com_watabou_coogee_ui_layouts_Grid(2);
		grid.setMargins(0, 10);
		_gthis.addCheckbox("Title", "city_name", true, grid, function (value) {
			com_watabou_mfcg_scenes_TownScene.instance.toggleOverlays();
		});
		_gthis.addCheckbox("Compass", "compass", true, grid, function (value) {
			com_watabou_mfcg_scenes_TownScene.instance.toggleOverlays();
		});
		_gthis.addCheckbox("Scale bar", "scale_bar", true, grid, function (value) {
			com_watabou_mfcg_scenes_TownScene.instance.toggleOverlays();
		});
		_gthis.addCheckbox("Emblem", "emblem", false, grid, function (value) {
			com_watabou_mfcg_scenes_TownScene.instance.toggleOverlays();
		});
		tab.add(grid);
		return tab;
	}
	, buildings: function () {
		var tab = new com_watabou_coogee_ui_layouts_VBox();
		tab.setMargins(10, 8);
		var grid = new com_watabou_coogee_ui_layouts_Grid(2);
		grid.setMargins(0, 8);
		var ddDisplayMode = null;
		this.addDropDown("Lots method", "lots_method", com_watabou_mfcg_Values.METHODS, grid, function (value) {
			switch (value) {
				case "Bisection":
					com_watabou_system_State.set("display_mode", ddDisplayMode.set_value("Simple"));
					break;
				case "Twisted":
					com_watabou_system_State.set("display_mode", ddDisplayMode.set_value("Lots"));
					break;
				case "Voronoi":
					com_watabou_system_State.set("display_mode", ddDisplayMode.set_value("Simple"));
					break;
			}
			com_watabou_mfcg_model_City.instance.updateLots();
			com_watabou_mfcg_scenes_TownScene.updateMap();
		});
		ddDisplayMode = this.addDropDown("Display mode", "display_mode", com_watabou_mfcg_Values.DISPLAY_MODES, grid, function (value) {
			com_watabou_mfcg_scenes_TownScene.updateMap();
		});
		this.addDropDown("Processing", "processing", com_watabou_mfcg_Values.PROCESSES, grid, function (value) {
			com_watabou_mfcg_model_City.instance.updateLots();
			com_watabou_mfcg_scenes_TownScene.updateMap();
		});
		tab.add(grid);
		this.addCheckbox("Filter lots", "no_triangles", false, tab, function (value) {
			com_watabou_mfcg_model_City.instance.updateLots();
			com_watabou_mfcg_scenes_TownScene.updateMap();
		});
		this.addCheckbox("Raised", "raised", true, tab, function (value) {
			com_watabou_mfcg_scenes_TownScene.updateMap();
		});
		return tab;
	}
	, outline: function () {
		var _gthis = this;
		var tab = new com_watabou_coogee_ui_layouts_VBox();
		tab.setMargins(10, 8);
		var checkboxes_h = Object.create(null);
		var onUpdate = null;
		var v = _gthis.addCheckbox("Outline buildings", "outline_buildings", true, tab, function (value) {
			if (onUpdate != null) {
				onUpdate(value);
			}
			com_watabou_mfcg_scenes_TownScene.updateMap();
		});
		checkboxes_h["outline_buildings"] = v;
		var onUpdate1 = function (value) {
			if (!value) {
				com_watabou_system_State.set("isolines", false);
				_gthis.chkIsolines.set_value(false);
			}
		};
		var v = _gthis.addCheckbox("Outline water", "outline_water", true, tab, function (value) {
			if (onUpdate1 != null) {
				onUpdate1(value);
			}
			com_watabou_mfcg_scenes_TownScene.updateMap();
		});
		checkboxes_h["outline_water"] = v;
		var onUpdate2 = null;
		var v = _gthis.addCheckbox("Outline roads", "outline_roads", true, tab, function (value) {
			if (onUpdate2 != null) {
				onUpdate2(value);
			}
			com_watabou_mfcg_scenes_TownScene.updateMap();
		});
		checkboxes_h["outline_roads"] = v;
		var onUpdate3 = null;
		var v = _gthis.addCheckbox("Outline trees", "outline_trees", true, tab, function (value) {
			if (onUpdate3 != null) {
				onUpdate3(value);
			}
			com_watabou_mfcg_scenes_TownScene.updateMap();
		});
		checkboxes_h["outline_trees"] = v;
		var btnToggle = new com_watabou_coogee_ui_Button("Toggle all", function () {
			var h = checkboxes_h;
			var inlStringMapValueIterator_h = h;
			var inlStringMapValueIterator_keys = Object.keys(h);
			var inlStringMapValueIterator_length = inlStringMapValueIterator_keys.length;
			var inlStringMapValueIterator_current = 0;
			var value = !inlStringMapValueIterator_h[inlStringMapValueIterator_keys[inlStringMapValueIterator_current++]].get_value();
			var h = checkboxes_h;
			var _g_h = h;
			var _g_keys = Object.keys(h);
			var _g_length = _g_keys.length;
			var _g_current = 0;
			while (_g_current < _g_length) {
				var key = _g_keys[_g_current++];
				var _g1_key = key;
				var _g1_value = _g_h[key];
				var id = _g1_key;
				var chk = _g1_value;
				chk.set_value(value);
				com_watabou_system_State.set(id, value);
			}
			if (!value) {
				com_watabou_system_State.set("isolines", false);
				_gthis.chkIsolines.set_value(false);
			}
			com_watabou_mfcg_scenes_TownScene.updateMap();
		});
		tab.add(btnToggle);
		return tab;
	}
	, text: function () {
		var tab = new com_watabou_coogee_ui_layouts_Grid(2);
		tab.setMargins(10, 8);
		this.addFont("Title", "font_title", com_watabou_mfcg_ui_Text.fontTitle, tab);
		this.addFont("Labels", "font_label", com_watabou_mfcg_ui_Text.fontLabel, tab);
		this.addFont("Legend", "font_legend", com_watabou_mfcg_ui_Text.fontLegend, tab);
		this.addFont("Pins", "font_pin", com_watabou_mfcg_ui_Text.fontPin, tab);
		this.addFont("Elements", "font_element", com_watabou_mfcg_ui_Text.fontElement, tab);
		return tab;
	}
	, misc: function () {
		var tab = new com_watabou_coogee_ui_layouts_VBox();
		tab.setMargins(10, 8);
		this.addCheckbox("Show trees", "show_trees", false, tab, function (value) {
			com_watabou_mfcg_scenes_TownScene.updateMap();
		});
		this.chkIsolines = this.addCheckbox("Water isolines", "isolines", true, tab, function (value) {
			com_watabou_mfcg_scenes_TownScene.updateMap();
		});
		this.addCheckbox("Solids", "draw_solids", true, tab, function (value) {
			com_watabou_mfcg_scenes_TownScene.updateMap();
		});
		var row = new com_watabou_coogee_ui_layouts_HBox();
		row.setMargins(0, 8);
		this.addDropDown("Towers", "towers", com_watabou_mfcg_Values.TOWER_SHAPES, row, function (value) {
			com_watabou_mfcg_scenes_TownScene.updateMap();
		});
		tab.add(row);
		return tab;
	}
	, __class__: com_watabou_mfcg_ui_forms_StyleForm
});
var com_watabou_mfcg_scenes_ViewScene = function () {
	com_watabou_mfcg_scenes_TownScene.call(this);
	this.btnMenu = new com_watabou_coogee_ui_Button("Menu", $bind(this, this.onMenu));
	if (!com_watabou_mfcg_Main.preview) {
		com_watabou_coogee_ui_UI.layer.addChild(this.btnMenu);
	}
	this.fader = com_watabou_processes_Tweener.create(1, $bind(this, this.onFadeOut));
};
$hxClasses["com.watabou.mfcg.scenes.ViewScene"] = com_watabou_mfcg_scenes_ViewScene;
com_watabou_mfcg_scenes_ViewScene.__name__ = "com.watabou.mfcg.scenes.ViewScene";
com_watabou_mfcg_scenes_ViewScene.__super__ = com_watabou_mfcg_scenes_TownScene;
com_watabou_mfcg_scenes_ViewScene.prototype = $extend(com_watabou_mfcg_scenes_TownScene.prototype, {
	activate: function () {
		com_watabou_mfcg_scenes_TownScene.prototype.activate.call(this);
		if (!com_watabou_mfcg_Main.preview) {
			com_watabou_coogee_ui_UI.layer.addChild(new com_watabou_mfcg_ui_Tooltip());
			com_watabou_mfcg_ui_Tooltip.instance.awake.add($bind(this, this.onAwake));
		}
		if (!com_watabou_mfcg_Main.preview) {
			com_watabou_mfcg_ui_forms_ToolForm.loadSaved(com_watabou_mfcg_scenes_ViewScene.tools);
		}
		var bp = com_watabou_mfcg_model_City.instance.bp;
		if (bp.export != null) {
			switch (bp.export.toLowerCase()) {
				case "json":
					com_watabou_mfcg_export_Export.asJSON();
					break;
				case "png":
					com_watabou_mfcg_export_Export.asPNG();
					break;
				case "svg":
					com_watabou_mfcg_export_Export.asSVG();
					break;
			}
			bp.export = null;
		}
	}
	, deactivate: function () {
		com_watabou_mfcg_scenes_TownScene.prototype.deactivate.call(this);
		com_watabou_coogee_ui_UI.layer.removeChild(this.btnMenu);
		if (com_watabou_mfcg_ui_Tooltip.instance != null) {
			com_watabou_coogee_ui_UI.layer.removeChild(com_watabou_mfcg_ui_Tooltip.instance);
		}
	}
	, onKeyEvent: function (keyCode, down) {
		if (down) {
			switch (keyCode) {
				case 13:
					this.buildNew();
					break;
				case 49:
					this.loadPreset("default");
					break;
				case 50:
					this.loadPreset("ink");
					break;
				case 51:
					this.loadPreset("bw");
					break;
				case 52:
					this.loadPreset("vivid");
					break;
				case 53:
					this.loadPreset("natural");
					break;
				case 54:
					this.loadPreset("modern");
					break;
				case 67:
					com_watabou_mfcg_scenes_TownScene.editColors();
					break;
				case 9: case 71:
					this.toggleWindow(com_watabou_mfcg_ui_forms_GenerateForm);
					break;
				case 76:
					this.toggleDistricts();
					break;
				case 83:
					this.toggleWindow(com_watabou_mfcg_ui_forms_StyleForm);
					break;
				case 84:
					this.toggleWindow(com_watabou_mfcg_ui_forms_TownForm);
					break;
				case 87:
					this.onWarp();
					break;
				default:
					com_watabou_mfcg_scenes_TownScene.prototype.onKeyEvent.call(this, keyCode, down);
			}
		}
	}
	, layout: function () {
		com_watabou_mfcg_scenes_TownScene.prototype.layout.call(this);
		this.btnMenu.set_x(com_watabou_coogee_ui_UI.layer.get_width() - this.btnMenu.get_width() - 2);
		this.btnMenu.set_y(2);
	}
	, onFadeOut: function (e) {
		this.btnMenu.set_alpha(com_watabou_mfcg_ui_Tooltip.instance.set_alpha(1 - e));
	}
	, onAwake: function (value) {
		if (value) {
			this.fader.stop();
			this.onFadeOut(0);
		} else {
			this.fader.start();
		}
	}
	, onMenu: function () {
		var context = com_watabou_mfcg_scenes_TownScene.context;
		this.onContext(context);
		context.show(this.btnMenu);
	}
	, onWarp: function () {
		com_watabou_coogee_Game.switchScene(com_watabou_mfcg_scenes_WarpScene);
	}
	, createOverlays: function () {
		com_watabou_mfcg_scenes_TownScene.prototype.createOverlays.call(this);
		this.overlays.push(this.title = new com_watabou_mfcg_scenes_overlays_TitleOverlay(this));
		this.addChild(this.title);
		this.overlays.push(this.emblem = new com_watabou_mfcg_scenes_overlays_EmblemOverlay(this));
		this.addChild(this.emblem);
	}
	, toggleOverlays: function (modified) {
		if (modified == null) {
			modified = false;
		}
		com_watabou_mfcg_scenes_TownScene.prototype.toggleOverlays.call(this, modified);
		this.title.set_visible(com_watabou_system_State.get("city_name", true) && !this.legend.get_visible());
		this.emblem.set_visible(com_watabou_system_State.get("emblem", false) && !this.legend.get_visible());
		if (com_watabou_mfcg_Main.preview) {
			var _g = 0;
			var _g1 = this.overlays;
			while (_g < _g1.length) {
				var overlay = _g1[_g];
				++_g;
				overlay.set_visible(false);
			}
		}
	}
	, arrangeOverlays: function () {
		com_watabou_mfcg_scenes_TownScene.prototype.arrangeOverlays.call(this);
		var tmp = this.emblem;
		var tmp1;
		if (this.legend.get_visible()) {
			switch (this.legend.get_position()._hx_index) {
				case 1: case 4:
					tmp1 = com_watabou_mfcg_scenes_overlays_Position.TOP_RIGHT;
					break;
				default:
					tmp1 = com_watabou_mfcg_scenes_overlays_Position.TOP_LEFT;
			}
		} else {
			tmp1 = com_watabou_mfcg_scenes_overlays_Position.TOP_LEFT;
		}
		tmp.set_position(tmp1);
	}
	, onMapContext: function (context) {
		var _gthis = this;
		context.group();
		var mapx = this.map.get_mouseX();
		var mapy = this.map.get_mouseY();
		if (com_watabou_system_State.get("landmarks") != "Hidden") {
			context.addItem("Add landmark", function () {
				_gthis.addLandmark(mapx, mapy);
			});
		}
		var point = new openfl_geom_Point(mapx, mapy);
		var patch = this.model.getPatch(point);
		if (patch != null && patch.isRerollable()) {
			context.addItem("Reroll geometry", function () {
				patch.reroll();
			});
			patch.ward.onContext(context, mapx, mapy);
		}
	}
	, onContext: function (context) {
		var _gthis = this;
		var $export = new com_watabou_coogee_ui_Menu();
		$export.addItem("PNG", com_watabou_mfcg_export_Export.asPNG);
		$export.addItem("SVG", com_watabou_mfcg_export_Export.asSVG);
		$export.addItem("JSON", com_watabou_mfcg_export_Export.asJSON);
		context.group();
		context.addItem("Procgen Arcana", $bind(this, this.arcana));
		context.addSeparator();
		context.addItem("New city", $bind(this, this.buildNew));
		context.addItem("Warp", $bind(this, this.onWarp));
		context.addItem("Color scheme...", com_watabou_mfcg_scenes_TownScene.editColors);
		context.addSubmenu("Export as...", $export);
		context.group();
		var addToggleable = function (name, cl) {
			context.addItem(name, function () {
				_gthis.toggleWindow(cl);
			}, com_watabou_coogee_ui_UI.findWidnow(cl) != null);
		};
		addToggleable("Generate", com_watabou_mfcg_ui_forms_GenerateForm);
		addToggleable("Settlement", com_watabou_mfcg_ui_forms_TownForm);
		addToggleable("Style", com_watabou_mfcg_ui_forms_StyleForm);
	}
	, toggleWindow: function (cl) {
		var wnd = com_watabou_coogee_ui_UI.findWidnow(cl);
		if (wnd == null) {
			var form = Type.createInstance(cl, []);
			com_watabou_coogee_ui_UI.showDialog(form);
			if (((form) instanceof com_watabou_mfcg_ui_forms_ToolForm)) {
				(js_Boot.__cast(form, com_watabou_mfcg_ui_forms_ToolForm)).restore();
			}
		} else {
			wnd.hide();
		}
	}
	, addLandmark: function (x, y) {
		com_watabou_coogee_ui_UI.showDialog(new com_watabou_mfcg_ui_forms_MarkerForm(this.model.addLandmark(x, y)));
	}
	, rerollGeometry: function (x, y) {
		var p = new openfl_geom_Point(x, y);
		var _g = 0;
		var _g1 = this.model.patches;
		while (_g < _g1.length) {
			var patch = _g1[_g];
			++_g;
			if (com_watabou_geom_polygons_PolyBounds.containsPoint(patch.shape, p)) {
				patch.reroll();
				break;
			}
		}
	}
	, buildNew: function () {
		new com_watabou_mfcg_model_City(com_watabou_mfcg_model_Blueprint.create(com_watabou_mfcg_model_City.nextSize, com_watabou_utils_Random.seed));
		com_watabou_coogee_Game.switchScene(com_watabou_mfcg_scenes_ViewScene);
	}
	, toggleDistricts: function () {
		var mode;
		switch (com_watabou_system_State.get("districts", "Curved")) {
			case "Curved":
				mode = "Legend";
				break;
			case "Hidden":
				mode = "Straight";
				break;
			case "Legend":
				mode = "Hidden";
				break;
			default:
				mode = "Curved";
		}
		com_watabou_system_State.set("districts", mode);
		this.toggleOverlays();
		var _g = 0;
		var _g1 = this.model.districts;
		while (_g < _g1.length) {
			var district = _g1[_g];
			++_g;
			district.updateGeometry();
		}
		this.layoutLabels();
	}
	, arcana: function () {
		var request = new openfl_net_URLRequest("https://watabou.github.io/");
		openfl_Lib.navigateToURL(request, "arcana");
	}
	, __class__: com_watabou_mfcg_scenes_ViewScene
});
var com_watabou_mfcg_scenes_WarpScene = function () {
	this.keyMap = new haxe_ds_IntMap();
	com_watabou_mfcg_scenes_TownScene.call(this);
	this.brush = new openfl_display_Shape();
	this.brush.set_cacheAsBitmap(true);
	this.addChild(this.brush);
	this.btnMenu = new com_watabou_coogee_ui_Button("Menu");
	this.btnMenu.click.add($bind(this, this.onMenu));
	this.addChild(this.btnMenu);
	this.nodePatches = new haxe_ds_ObjectMap();
	this.prevState = new haxe_ds_ObjectMap();
	var _g = 0;
	var _g1 = this.model.patches;
	while (_g < _g1.length) {
		var patch = _g1[_g];
		++_g;
		var _g2 = 0;
		var _g3 = patch.shape;
		while (_g2 < _g3.length) {
			var v = _g3[_g2];
			++_g2;
			if (this.nodePatches.h.__keys__[v.__id__] == null) {
				var v1 = [patch];
				this.nodePatches.set(v, v1);
				var this1 = this.prevState;
				var v2 = v.clone();
				this1.set(v, v2);
			} else {
				this.nodePatches.h[v.__id__].push(patch);
			}
		}
	}
	var this1 = this.keyMap;
	var v = new com_watabou_mfcg_scenes_tools_DisplaceTool(this);
	this1.h[68] = v;
	var this1 = this.keyMap;
	var v = new com_watabou_mfcg_scenes_tools_RotateTool(this);
	this1.h[82] = v;
	var this1 = this.keyMap;
	var v = new com_watabou_mfcg_scenes_tools_LiquifyTool(this);
	this1.h[76] = v;
	var this1 = this.keyMap;
	var v = new com_watabou_mfcg_scenes_tools_RelaxTool(this);
	this1.h[88] = v;
	var this1 = this.keyMap;
	var v = new com_watabou_mfcg_scenes_tools_BloatTool(this);
	this1.h[66] = v;
	var this1 = this.keyMap;
	var v = new com_watabou_mfcg_scenes_tools_PinchTool(this);
	this1.h[80] = v;
	var this1 = this.keyMap;
	var v = new com_watabou_mfcg_scenes_tools_MeasureTool(this);
	this1.h[77] = v;
	var this1 = this.keyMap;
	var v = new com_watabou_mfcg_scenes_tools_EqualizeTool(this);
	this1.h[69] = v;
	if (com_watabou_mfcg_scenes_WarpScene.lastTool == null) {
		this.switchTool(this.keyMap.h[68]);
	} else {
		var tool = this.keyMap.iterator();
		while (tool.hasNext()) {
			var tool1 = tool.next();
			if (js_Boot.getClass(tool1) == js_Boot.getClass(com_watabou_mfcg_scenes_WarpScene.lastTool)) {
				this.switchTool(tool1);
				break;
			}
		}
	}
	this.mesh = new openfl_display_Sprite();
};
$hxClasses["com.watabou.mfcg.scenes.WarpScene"] = com_watabou_mfcg_scenes_WarpScene;
com_watabou_mfcg_scenes_WarpScene.__name__ = "com.watabou.mfcg.scenes.WarpScene";
com_watabou_mfcg_scenes_WarpScene.__super__ = com_watabou_mfcg_scenes_TownScene;
com_watabou_mfcg_scenes_WarpScene.prototype = $extend(com_watabou_mfcg_scenes_TownScene.prototype, {
	onEsc: function () {
		this.onDiscard();
	}
	, onKeyEvent: function (keyCode, pressed) {
		if (!pressed || !this.tool.onKey(keyCode)) {
			if (keyCode == 13) {
				if (pressed) {
					this.onSave();
				}
			} else if (pressed && this.keyMap.h.hasOwnProperty(keyCode)) {
				this.switchTool(this.keyMap.h[keyCode]);
			} else {
				com_watabou_mfcg_scenes_TownScene.prototype.onKeyEvent.call(this, keyCode, pressed);
			}
		}
	}
	, switchTool: function (tool) {
		com_watabou_mfcg_scenes_WarpScene.lastTool = this.tool = tool;
		if (this.map != null) {
			tool.activate();
			tool.onMove(this.map.get_mouseX(), this.map.get_mouseY());
			com_watabou_coogee_ui_Toast.show(tool.getName());
		}
	}
	, layout: function () {
		com_watabou_mfcg_scenes_TownScene.prototype.layout.call(this);
		this.brush.set_x(this.get_mouseX());
		this.brush.set_y(this.get_mouseY());
		this.tool.activate();
		this.btnMenu.set_x(this.rWidth - this.btnMenu.get_width() - 2);
		this.btnMenu.set_y(2);
	}
	, recreateMap: function () {
		com_watabou_mfcg_scenes_TownScene.prototype.recreateMap.call(this);
		this.map.addChild(this.mesh);
		this.map.mouseChildren = false;
	}
	, createOverlays: function () {
		com_watabou_mfcg_scenes_TownScene.prototype.createOverlays.call(this);
		var _g = 0;
		var _g1 = this.overlays;
		while (_g < _g1.length) {
			var o = _g1[_g];
			++_g;
			o.mouseEnabled = false;
			o.mouseChildren = false;
		}
	}
	, activate: function () {
		com_watabou_mfcg_scenes_TownScene.prototype.activate.call(this);
		var _g = [];
		var _g1 = 0;
		var _g2 = com_watabou_mfcg_scenes_ViewScene.tools;
		while (_g1 < _g2.length) {
			var cl = _g2[_g1];
			++_g1;
			var wnd = com_watabou_coogee_ui_UI.findWidnow(cl);
			if (wnd != null) {
				_g.push(wnd.content);
			}
		}
		var tools = _g;
		com_watabou_coogee_ui_UI.wipe();
		var _g = 0;
		while (_g < tools.length) {
			var tool = tools[_g];
			++_g;
			tool.forceDisplay();
		}
		this.addEventListener("mouseWheel", $bind(this, this.onMouseWheel));
		this.addEventListener("mouseDown", $bind(this, this.onMouseDown));
		this.stage.addEventListener("mouseUp", $bind(this, this.onMouseUp));
		this.stage.addEventListener("mouseMove", $bind(this, this.onMouseMove));
		this.brush.set_x(this.get_mouseX());
		this.brush.set_y(this.get_mouseY());
		com_watabou_coogee_ui_Toast.show(this.tool.getName());
	}
	, deactivate: function () {
		this.removeEventListener("mouseWheel", $bind(this, this.onMouseWheel));
		this.removeEventListener("mouseDown", $bind(this, this.onMouseDown));
		this.stage.removeEventListener("mouseUp", $bind(this, this.onMouseUp));
		this.stage.removeEventListener("mouseMove", $bind(this, this.onMouseMove));
		com_watabou_mfcg_scenes_TownScene.prototype.deactivate.call(this);
	}
	, updateBrush: function (radius, inner) {
		if (inner == null) {
			inner = 0.5;
		}
		this.brush.get_graphics().clear();
		if (radius > 0) {
			this.brush.get_graphics().lineStyle(1., 52224);
			this.brush.get_graphics().drawCircle(0, 0, radius * this.map.get_scaleX());
			this.brush.get_graphics().lineStyle(2.0, 52224);
			this.brush.get_graphics().drawCircle(0, 0, radius * this.map.get_scaleX() * inner);
		}
	}
	, onMouseWheel: function (e) {
		this.tool.onWheel(this.map.get_mouseX(), this.map.get_mouseY(), e.delta);
	}
	, onMouseMove: function (e) {
		this.brush.set_x(this.get_mouseX());
		this.brush.set_y(this.get_mouseY());
		var mx = this.map.get_mouseX();
		var my = this.map.get_mouseY();
		if (this.down) {
			this.tool.onDrag(mx, my);
		} else {
			this.tool.onMove(mx, my);
		}
	}
	, onMouseDown: function (e) {
		this.down = true;
		this.tool.onPress(this.map.get_mouseX(), this.map.get_mouseY());
	}
	, onMouseUp: function (e) {
		this.tool.onRelease();
		this.down = false;
	}
	, onMenu: function () {
		var context = com_watabou_mfcg_scenes_TownScene.context;
		this.onContext(context);
		context.show(this.btnMenu);
	}
	, onContext: function (context) {
		var _gthis = this;
		var tool = this.keyMap.iterator();
		while (tool.hasNext()) {
			var tool1 = [tool.next()];
			context.addItem(tool1[0].getName(), (function (tool) {
				return function () {
					_gthis.switchTool(tool[0]);
				};
			})(tool1), tool1[0] == this.tool);
		}
		context.group();
		context.addItem("Apply", $bind(this, this.onSave));
		context.addItem("Discard", $bind(this, this.onDiscard));
	}
	, submit: function (patches, updateTideline) {
		this.model.updateGeometry(patches);
		if (updateTideline) {
			this.model.tideLine = null;
		}
		this.drawMap();
		this.layoutLabels();
		this.model.updateLandmarks();
		this.markers.setSize(this.rWidth, this.rHeight);
	}
	, clearMesh: function () {
		var _g = 0;
		var _g1 = this.mesh.get_numChildren();
		while (_g < _g1) {
			var i = _g++;
			com_watabou_mfcg_scenes_WarpScene.cache.push(this.mesh.getChildAt(i));
		}
		this.mesh.removeChildren();
	}
	, drawEdge: function (v1, v2, thickness) {
		var bmp = com_watabou_mfcg_scenes_WarpScene.cache.length > 0 ? com_watabou_mfcg_scenes_WarpScene.cache.pop() : new openfl_display_Bitmap(com_watabou_mfcg_scenes_WarpScene.pixel);
		bmp.set_x(v1.x);
		bmp.set_y(v1.y);
		bmp.set_rotation(Math.atan2(v2.y - v1.y, v2.x - v1.x) / Math.PI * 180);
		bmp.set_scaleX(openfl_geom_Point.distance(v1, v2));
		if (thickness >= 0.5) {
			bmp.set_scaleY(thickness * com_watabou_mfcg_mapping_Style.lineInvScale);
			bmp.set_alpha(1);
		} else {
			bmp.set_scaleY(0.5 * com_watabou_mfcg_mapping_Style.lineInvScale);
			bmp.set_alpha(thickness / 0.5);
		}
		this.mesh.addChild(bmp);
	}
	, drawNode: function (v, radius) {
		radius *= com_watabou_mfcg_mapping_Style.lineInvScale;
		var bmp = com_watabou_mfcg_scenes_WarpScene.cache.length > 0 ? com_watabou_mfcg_scenes_WarpScene.cache.pop() : new openfl_display_Bitmap(com_watabou_mfcg_scenes_WarpScene.pixel);
		bmp.set_alpha(1);
		bmp.set_rotation(0);
		bmp.set_scaleX(bmp.set_scaleY(radius * 2));
		bmp.set_x(v.x - radius);
		bmp.set_y(v.y - radius);
		this.mesh.addChild(bmp);
	}
	, getBmp: function () {
		if (com_watabou_mfcg_scenes_WarpScene.cache.length > 0) {
			return com_watabou_mfcg_scenes_WarpScene.cache.pop();
		} else {
			return new openfl_display_Bitmap(com_watabou_mfcg_scenes_WarpScene.pixel);
		}
	}
	, onSave: function () {
		this.model.updateDimensions();
		com_watabou_coogee_Game.switchScene(com_watabou_mfcg_scenes_ViewScene);
	}
	, onDiscard: function () {
		var v = this.prevState.keys();
		while (v.hasNext()) {
			var v1 = v.next();
			var prev = this.prevState.h[v1.__id__];
			v1.setTo(prev.x, prev.y);
		}
		this.model.updateGeometry(this.model.patches);
		this.model.tideLine = null;
		com_watabou_coogee_Game.switchScene(com_watabou_mfcg_scenes_ViewScene);
	}
	, __class__: com_watabou_mfcg_scenes_WarpScene
});
var com_watabou_mfcg_scenes_overlays_Compass = function (radius) {
	openfl_display_Sprite.call(this);
	if (com_watabou_mfcg_scenes_overlays_Compass.convexity == 0.0) {
		this.reroll();
	}
	this.radius = radius;
	this.update();
	this.label = new openfl_display_Sprite();
	this.addChild(this.label);
	var format = com_watabou_mfcg_ui_Text.getFormat("font_element", com_watabou_mfcg_ui_Text.fontElement, com_watabou_mfcg_mapping_Style.colorDark, 1.28571428571428581);
	this.tf = com_watabou_mfcg_ui_Text.get("N", format);
	this.tf.mouseEnabled = false;
	this.label.addChild(this.tf);
	this.addEventListener("click", $bind(this, this.reset));
	this.addEventListener("mouseWheel", $bind(this, this.rotate));
};
$hxClasses["com.watabou.mfcg.scenes.overlays.Compass"] = com_watabou_mfcg_scenes_overlays_Compass;
com_watabou_mfcg_scenes_overlays_Compass.__name__ = "com.watabou.mfcg.scenes.overlays.Compass";
com_watabou_mfcg_scenes_overlays_Compass.__super__ = openfl_display_Sprite;
com_watabou_mfcg_scenes_overlays_Compass.prototype = $extend(openfl_display_Sprite.prototype, {
	reroll: function () {
		com_watabou_mfcg_scenes_overlays_Compass.convexity = 0.1 + 0.2 * ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647);
		var chance = 0.9;
		if (chance == null) {
			chance = 0.5;
		}
		com_watabou_mfcg_scenes_overlays_Compass.secondary = (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance ? com_watabou_mfcg_scenes_overlays_Compass.convexity + (1 - com_watabou_mfcg_scenes_overlays_Compass.convexity) * (0.1 + 0.9 * ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647)) : 0.0;
		var chance = 0.8;
		if (chance == null) {
			chance = 0.5;
		}
		if ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance) {
			com_watabou_mfcg_scenes_overlays_Compass.mainRing = com_watabou_mfcg_scenes_overlays_Compass.convexity + (1 - com_watabou_mfcg_scenes_overlays_Compass.convexity) * (((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 3);
			com_watabou_mfcg_scenes_overlays_Compass.auxRing = (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647;
		} else {
			com_watabou_mfcg_scenes_overlays_Compass.mainRing = com_watabou_mfcg_scenes_overlays_Compass.auxRing = 0.0;
		}
		var chance = Math.pow(1 - com_watabou_mfcg_scenes_overlays_Compass.convexity, 2);
		if (chance == null) {
			chance = 0.5;
		}
		if ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance) {
			com_watabou_mfcg_scenes_overlays_Compass.north = 1.3 + 0.6 * (((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 3);
			var chance = 0.7;
			if (chance == null) {
				chance = 0.5;
			}
			com_watabou_mfcg_scenes_overlays_Compass.south = (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 < chance ? 1 + (com_watabou_mfcg_scenes_overlays_Compass.north - 1) * (((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 3) : 1.0;
		} else {
			com_watabou_mfcg_scenes_overlays_Compass.north = com_watabou_mfcg_scenes_overlays_Compass.south = 1.0;
		}
	}
	, update: function () {
		var g = this.get_graphics();
		g.clear();
		g.beginFill(16711680, 0.0);
		g.drawCircle(0, 0, this.radius);
		g.endFill();
		if (com_watabou_mfcg_scenes_overlays_Compass.mainRing > 0) {
			var unscaled = false;
			if (unscaled == null) {
				unscaled = true;
			}
			g.lineStyle(com_watabou_mfcg_mapping_Style.getStrokeWidth(com_watabou_mfcg_mapping_Style.strokeThick, unscaled), com_watabou_mfcg_mapping_Style.colorDark);
			g.drawCircle(0, 0, this.radius * com_watabou_mfcg_scenes_overlays_Compass.mainRing);
			var unscaled = false;
			if (unscaled == null) {
				unscaled = true;
			}
			g.lineStyle(com_watabou_mfcg_mapping_Style.getStrokeWidth(com_watabou_mfcg_mapping_Style.strokeNormal, unscaled), com_watabou_mfcg_mapping_Style.colorDark);
			g.drawCircle(0, 0, this.radius * com_watabou_mfcg_scenes_overlays_Compass.auxRing);
			g.endFill();
		}
		this.drawStar(com_watabou_mfcg_scenes_overlays_Compass.secondary, com_watabou_mfcg_scenes_overlays_Compass.convexity * com_watabou_mfcg_scenes_overlays_Compass.secondary, 0.5);
		this.drawStar(1, com_watabou_mfcg_scenes_overlays_Compass.convexity, 0.0);
	}
	, drawStar: function (r1, r2, ph) {
		var g = this.get_graphics();
		var poly = [];
		poly.push(openfl_geom_Point.polar(this.radius * r1, Math.PI / 2 * ph));
		poly.push(openfl_geom_Point.polar(this.radius * r2, Math.PI / 2 * (ph + 0.5)));
		poly.push(openfl_geom_Point.polar(this.radius * r1, Math.PI / 2 * (1 + ph)));
		poly.push(openfl_geom_Point.polar(this.radius * r2, Math.PI / 2 * (1 + ph + 0.5)));
		poly.push(openfl_geom_Point.polar(this.radius * r1, Math.PI / 2 * (2 + ph)));
		poly.push(openfl_geom_Point.polar(this.radius * r2, Math.PI / 2 * (2 + ph + 0.5)));
		poly.push(openfl_geom_Point.polar(this.radius * r1, Math.PI / 2 * (3 + ph)));
		poly.push(openfl_geom_Point.polar(this.radius * r2, Math.PI / 2 * (3 + ph + 0.5)));
		g.beginFill(com_watabou_mfcg_mapping_Style.colorDark);
		com_watabou_utils_GraphicsExtender.drawPolygon(g, [poly[0], poly[1], com_watabou_mfcg_scenes_overlays_Compass.o]);
		com_watabou_utils_GraphicsExtender.drawPolygon(g, [poly[2], poly[3], com_watabou_mfcg_scenes_overlays_Compass.o]);
		com_watabou_utils_GraphicsExtender.drawPolygon(g, [poly[4], poly[5], com_watabou_mfcg_scenes_overlays_Compass.o]);
		com_watabou_utils_GraphicsExtender.drawPolygon(g, [poly[6], poly[7], com_watabou_mfcg_scenes_overlays_Compass.o]);
		g.beginFill(com_watabou_mfcg_mapping_Style.colorLight);
		com_watabou_utils_GraphicsExtender.drawPolygon(g, [poly[1], poly[2 % 8], com_watabou_mfcg_scenes_overlays_Compass.o]);
		com_watabou_utils_GraphicsExtender.drawPolygon(g, [poly[3], poly[4 % 8], com_watabou_mfcg_scenes_overlays_Compass.o]);
		com_watabou_utils_GraphicsExtender.drawPolygon(g, [poly[5], poly[6 % 8], com_watabou_mfcg_scenes_overlays_Compass.o]);
		com_watabou_utils_GraphicsExtender.drawPolygon(g, [poly[7], poly[8 % 8], com_watabou_mfcg_scenes_overlays_Compass.o]);
		g.endFill();
		var unscaled = false;
		if (unscaled == null) {
			unscaled = true;
		}
		g.lineStyle(com_watabou_mfcg_mapping_Style.getStrokeWidth(com_watabou_mfcg_mapping_Style.strokeNormal, unscaled), com_watabou_mfcg_mapping_Style.colorDark);
		com_watabou_utils_GraphicsExtender.drawPolygon(g, poly);
		g.endFill();
	}
	, updateLabel: function (dir) {
		this.tf.set_text(["N", "W", "S", "E"][dir]);
		this.tf.set_x(-this.tf.get_width() / 2);
		this.tf.set_y(-this.tf.get_height() + 2);
		this.label.set_rotation(-90 * dir);
		var pos = openfl_geom_Point.polar(this.radius, -(dir + 1) * Math.PI / 2);
		this.label.set_x(pos.x);
		this.label.set_y(pos.y);
	}
	, reset: function (e) {
		if (e.ctrlKey || e.commandKey) {
			this.updateNorth(com_watabou_mfcg_model_City.instance.north = 0.0);
		} else if (e.shiftKey) {
			this.reroll();
			this.update();
		}
	}
	, rotate: function (e) {
		var tmp = this.get_rotation();
		var value = e.delta;
		this.set_rotation(tmp + (value == 0 ? 0 : value < 0 ? -1 : 1) * 10);
		this.updateLabel((this.get_rotation() + 405) % 360 / 90 | 0);
		com_watabou_mfcg_model_City.instance.north = this.get_rotation();
	}
	, updateNorth: function (north) {
		this.set_rotation(north);
		this.updateLabel((this.get_rotation() + 405) % 360 / 90 | 0);
	}
	, __class__: com_watabou_mfcg_scenes_overlays_Compass
});
var com_watabou_mfcg_scenes_overlays_Overlay = function (scene) {
	var _gthis = this;
	com_watabou_coogee_ui_View.call(this);
	this.scene = scene;
	this.addEventListener("removedFromStage", function (e) {
		_gthis.onDestroy();
	});
	this.addEventListener("rightClick", function (e) {
		_gthis.onContext(com_watabou_mfcg_scenes_TownScene.context);
	});
	com_watabou_mfcg_model_ModelDispatcher.newModel.add($bind(this, this.onNewModel));
};
$hxClasses["com.watabou.mfcg.scenes.overlays.Overlay"] = com_watabou_mfcg_scenes_overlays_Overlay;
com_watabou_mfcg_scenes_overlays_Overlay.__name__ = "com.watabou.mfcg.scenes.overlays.Overlay";
com_watabou_mfcg_scenes_overlays_Overlay.__super__ = com_watabou_coogee_ui_View;
com_watabou_mfcg_scenes_overlays_Overlay.prototype = $extend(com_watabou_coogee_ui_View.prototype, {
	map2layer: function (map, p) {
		return this.globalToLocal(map.localToGlobal(p));
	}
	, layer2map: function (map, p) {
		return map.globalToLocal(this.localToGlobal(p));
	}
	, update: function (model) {
		this.model = model;
	}
	, exportPNG: function (state) {
	}
	, onContext: function (context) {
	}
	, onNewModel: function (model) {
		this.model = model;
	}
	, onDestroy: function () {
		com_watabou_mfcg_model_ModelDispatcher.newModel.remove($bind(this, this.onNewModel));
	}
	, get_position: function () {
		return com_watabou_mfcg_scenes_overlays_Position.UNDEFINED;
	}
	, set_position: function (value) {
		return com_watabou_mfcg_scenes_overlays_Position.UNDEFINED;
	}
	, __class__: com_watabou_mfcg_scenes_overlays_Overlay
	, __properties__: $extend(com_watabou_coogee_ui_View.prototype.__properties__, { set_position: "set_position", get_position: "get_position" })
});
var com_watabou_mfcg_scenes_overlays_CompassOverlay = function (scene) {
	this.pos = com_watabou_mfcg_scenes_overlays_Position.BOTTOM_LEFT;
	this.compass = new com_watabou_mfcg_scenes_overlays_Compass(com_watabou_mfcg_scenes_overlays_CompassOverlay.RADIUS);
	com_watabou_mfcg_scenes_overlays_Overlay.call(this, scene);
	this.addChild(this.compass);
};
$hxClasses["com.watabou.mfcg.scenes.overlays.CompassOverlay"] = com_watabou_mfcg_scenes_overlays_CompassOverlay;
com_watabou_mfcg_scenes_overlays_CompassOverlay.__name__ = "com.watabou.mfcg.scenes.overlays.CompassOverlay";
com_watabou_mfcg_scenes_overlays_CompassOverlay.__super__ = com_watabou_mfcg_scenes_overlays_Overlay;
com_watabou_mfcg_scenes_overlays_CompassOverlay.prototype = $extend(com_watabou_mfcg_scenes_overlays_Overlay.prototype, {
	update: function (model) {
		com_watabou_mfcg_scenes_overlays_Overlay.prototype.update.call(this, model);
		this.compass.updateNorth(model.north);
	}
	, layout: function () {
		switch (this.pos._hx_index) {
			case 1:
				this.compass.set_x(this.compass.radius + com_watabou_mfcg_scenes_overlays_Overlay.MARGIN);
				this.compass.set_y(this.compass.radius + com_watabou_mfcg_scenes_overlays_Overlay.MARGIN);
				break;
			case 2:
				this.compass.set_x(this.rWidth - this.compass.radius - com_watabou_mfcg_scenes_overlays_Overlay.MARGIN);
				this.compass.set_y(this.compass.radius + com_watabou_mfcg_scenes_overlays_Overlay.MARGIN);
				break;
			case 3:
				this.compass.set_x(this.compass.radius + com_watabou_mfcg_scenes_overlays_Overlay.MARGIN);
				this.compass.set_y(this.rHeight - this.compass.radius - com_watabou_mfcg_scenes_overlays_Overlay.MARGIN);
				break;
			case 4:
				this.compass.set_x(this.rWidth - this.compass.radius - com_watabou_mfcg_scenes_overlays_Overlay.MARGIN);
				this.compass.set_y(this.rHeight - this.compass.radius - com_watabou_mfcg_scenes_overlays_Overlay.MARGIN);
				break;
			default:
		}
	}
	, get_position: function () {
		return this.pos;
	}
	, set_position: function (pos) {
		if (this.pos != pos) {
			this.pos = pos;
			this.layout();
		}
		return pos;
	}
	, onContext: function (context) {
		var _gthis = this;
		context.addItem("Reroll", function () {
			_gthis.compass.reroll();
			_gthis.compass.update();
		});
		context.addItem("Reset", function () {
			_gthis.compass.updateNorth(_gthis.model.north = 0.0);
		});
		context.addItem("Hide", function () {
			com_watabou_system_State.set("compass", false);
		});
	}
	, __class__: com_watabou_mfcg_scenes_overlays_CompassOverlay
});
var com_watabou_mfcg_scenes_overlays_Emblem = function () {
	this.ready = new msignal_Signal0();
	var _gthis = this;
	com_watabou_coogee_ui_View.call(this);
	this.bmp = new openfl_display_Bitmap(null, null, true);
	this.addChild(this.bmp);
	com_watabou_mfcg_scenes_overlays_Emblem.seed = com_watabou_system_State.get("emblem_seed", 1);
	com_watabou_mfcg_scenes_overlays_Emblem.coa = com_watabou_system_State.get("emblem_coa");
	com_watabou_mfcg_scenes_overlays_Emblem.updated.add($bind(this, this.onUpdated));
	com_watabou_mfcg_scenes_overlays_Emblem.setHiRes.add($bind(this, this.onResolution));
	this.addEventListener("removed", function (e) {
		com_watabou_mfcg_scenes_overlays_Emblem.updated.remove($bind(_gthis, _gthis.onUpdated));
		com_watabou_mfcg_scenes_overlays_Emblem.setHiRes.remove($bind(_gthis, _gthis.onResolution));
	});
	if (!com_watabou_mfcg_scenes_overlays_Emblem.loading) {
		if (com_watabou_mfcg_scenes_overlays_Emblem.loRes == null) {
			com_watabou_mfcg_scenes_overlays_Emblem.loadLo();
		} else {
			this.onUpdated();
		}
	}
};
$hxClasses["com.watabou.mfcg.scenes.overlays.Emblem"] = com_watabou_mfcg_scenes_overlays_Emblem;
com_watabou_mfcg_scenes_overlays_Emblem.__name__ = "com.watabou.mfcg.scenes.overlays.Emblem";
com_watabou_mfcg_scenes_overlays_Emblem.loadLo = function () {
	com_watabou_mfcg_scenes_overlays_Emblem.loading = true;
	var link = com_watabou_mfcg_scenes_overlays_Emblem.coa == null ? "" + com_watabou_mfcg_scenes_overlays_Emblem.ARMORIA + "/png/" + com_watabou_mfcg_scenes_overlays_Emblem.LO_RES + "/" + com_watabou_mfcg_scenes_overlays_Emblem.seed : "" + com_watabou_mfcg_scenes_overlays_Emblem.ARMORIA + "/?format=png&size=" + com_watabou_mfcg_scenes_overlays_Emblem.LO_RES + "&coa=" + com_watabou_mfcg_scenes_overlays_Emblem.coa;
	openfl_display_BitmapData.loadFromFile(link).onComplete(com_watabou_mfcg_scenes_overlays_Emblem.onLoadLo).onError(com_watabou_mfcg_scenes_overlays_Emblem.onError);
};
com_watabou_mfcg_scenes_overlays_Emblem.onError = function (response) {
	com_watabou_mfcg_scenes_overlays_Emblem.loading = false;
	com_watabou_mfcg_scenes_overlays_Emblem.setCOA(null);
};
com_watabou_mfcg_scenes_overlays_Emblem.onLoadLo = function (bmpData) {
	com_watabou_mfcg_scenes_overlays_Emblem.loading = false;
	if (com_watabou_mfcg_scenes_overlays_Emblem.loRes != null) {
		com_watabou_mfcg_scenes_overlays_Emblem.loRes.dispose();
	}
	com_watabou_mfcg_scenes_overlays_Emblem.loRes = bmpData;
	if (com_watabou_mfcg_scenes_overlays_Emblem.hiRes == null) {
		com_watabou_mfcg_scenes_overlays_Emblem.loadHi();
	}
	if (com_watabou_mfcg_scenes_overlays_Emblem.svg == null) {
		com_watabou_mfcg_scenes_overlays_Emblem.loadSvg();
	}
	com_watabou_mfcg_scenes_overlays_Emblem.updated.dispatch();
};
com_watabou_mfcg_scenes_overlays_Emblem.loadHi = function () {
	var link = com_watabou_mfcg_scenes_overlays_Emblem.coa == null ? "" + com_watabou_mfcg_scenes_overlays_Emblem.ARMORIA + "/png/" + com_watabou_mfcg_scenes_overlays_Emblem.HI_RES + "/" + com_watabou_mfcg_scenes_overlays_Emblem.seed : "" + com_watabou_mfcg_scenes_overlays_Emblem.ARMORIA + "/?format=png&size=" + com_watabou_mfcg_scenes_overlays_Emblem.HI_RES + "&coa=" + com_watabou_mfcg_scenes_overlays_Emblem.coa;
	openfl_display_BitmapData.loadFromFile(link).onComplete(com_watabou_mfcg_scenes_overlays_Emblem.onLoadHi);
};
com_watabou_mfcg_scenes_overlays_Emblem.onLoadHi = function (bmpData) {
	if (com_watabou_mfcg_scenes_overlays_Emblem.hiRes != null) {
		com_watabou_mfcg_scenes_overlays_Emblem.hiRes.dispose();
	}
	com_watabou_mfcg_scenes_overlays_Emblem.hiRes = bmpData;
};
com_watabou_mfcg_scenes_overlays_Emblem.loadSvg = function () {
	var link = com_watabou_mfcg_scenes_overlays_Emblem.coa == null ? "" + com_watabou_mfcg_scenes_overlays_Emblem.ARMORIA + "/svg/" + com_watabou_mfcg_scenes_overlays_Emblem.LO_RES + "/" + com_watabou_mfcg_scenes_overlays_Emblem.seed : "" + com_watabou_mfcg_scenes_overlays_Emblem.ARMORIA + "/?format=svg&size=" + com_watabou_mfcg_scenes_overlays_Emblem.LO_RES + "&coa=" + com_watabou_mfcg_scenes_overlays_Emblem.coa;
	var loader = new openfl_net_URLLoader();
	loader.addEventListener("complete", function (e) {
		com_watabou_mfcg_scenes_overlays_Emblem.onLoadSvg(loader.data);
	});
	loader.addEventListener("ioError", com_watabou_mfcg_scenes_overlays_Emblem.onIOError);
	loader.load(new openfl_net_URLRequest(link));
};
com_watabou_mfcg_scenes_overlays_Emblem.onLoadSvg = function (data) {
	com_watabou_mfcg_scenes_overlays_Emblem.svg = Xml.parse(data).firstElement();
};
com_watabou_mfcg_scenes_overlays_Emblem.onIOError = function (e) {
	if (e.errorID == 0) {
		com_watabou_mfcg_scenes_overlays_Emblem.onLoadSvg(e.text);
	}
};
com_watabou_mfcg_scenes_overlays_Emblem.setCOA = function (coa) {
	var tmp = coa != null;
	com_watabou_system_State.set("emblem_coa", com_watabou_mfcg_scenes_overlays_Emblem.coa = coa);
	com_watabou_mfcg_scenes_overlays_Emblem.loRes = com_watabou_mfcg_scenes_overlays_Emblem.hiRes = null;
	com_watabou_mfcg_scenes_overlays_Emblem.svg = null;
	com_watabou_mfcg_scenes_overlays_Emblem.loadLo();
};
com_watabou_mfcg_scenes_overlays_Emblem.setResoltion = function (high) {
	com_watabou_mfcg_scenes_overlays_Emblem.setHiRes.dispatch(high);
};
com_watabou_mfcg_scenes_overlays_Emblem.__super__ = com_watabou_coogee_ui_View;
com_watabou_mfcg_scenes_overlays_Emblem.prototype = $extend(com_watabou_coogee_ui_View.prototype, {
	customize: function () {
		if (com_watabou_coogee_ui_UI.findWidnow(com_watabou_mfcg_ui_forms_EmblemForm) == null) {
			com_watabou_coogee_ui_UI.showDialog(new com_watabou_mfcg_ui_forms_EmblemForm(this));
		}
	}
	, onUpdated: function () {
		this.bmp.set_bitmapData(com_watabou_mfcg_scenes_overlays_Emblem.loRes);
		this.rWidth = this.bmp.get_width();
		this.rHeight = this.bmp.get_height();
		this.ready.dispatch();
	}
	, reroll: function () {
		com_watabou_system_State.set("emblem_seed", com_watabou_mfcg_scenes_overlays_Emblem.seed = new Date().getTime() % 2147483647 | 0);
		com_watabou_system_State.set("emblem_coa", com_watabou_mfcg_scenes_overlays_Emblem.coa = null);
		com_watabou_mfcg_scenes_overlays_Emblem.loRes = com_watabou_mfcg_scenes_overlays_Emblem.hiRes = null;
		com_watabou_mfcg_scenes_overlays_Emblem.svg = null;
		com_watabou_mfcg_scenes_overlays_Emblem.loadLo();
	}
	, onResolution: function (high) {
		if (high && com_watabou_mfcg_scenes_overlays_Emblem.hiRes != null) {
			this.bmp.set_bitmapData(com_watabou_mfcg_scenes_overlays_Emblem.hiRes);
			this.bmp.set_scaleX(this.bmp.set_scaleY(com_watabou_mfcg_scenes_overlays_Emblem.LO_RES / com_watabou_mfcg_scenes_overlays_Emblem.HI_RES));
			this.bmp.smoothing = true;
		} else {
			this.bmp.set_bitmapData(com_watabou_mfcg_scenes_overlays_Emblem.loRes);
			this.bmp.set_scaleX(this.bmp.set_scaleY(1));
			if (high) {
				com_watabou_coogee_ui_Toast.show("High resolution emblem is not loaded");
			}
		}
	}
	, getSVG: function (svg) {
		var group = com_watabou_formats_SVG.group("emblem");
		if (com_watabou_mfcg_scenes_overlays_Emblem.svg == null) {
			com_watabou_coogee_ui_Toast.show("Emblem svg is not loaded");
		} else {
			group.addChild(com_watabou_mfcg_scenes_overlays_Emblem.svg);
		}
		return group;
	}
	, __class__: com_watabou_mfcg_scenes_overlays_Emblem
});
var com_watabou_mfcg_scenes_overlays_Position = $hxEnums["com.watabou.mfcg.scenes.overlays.Position"] = {
	__ename__: "com.watabou.mfcg.scenes.overlays.Position", __constructs__: null
	, UNDEFINED: { _hx_name: "UNDEFINED", _hx_index: 0, __enum__: "com.watabou.mfcg.scenes.overlays.Position", toString: $estr }
	, TOP_LEFT: { _hx_name: "TOP_LEFT", _hx_index: 1, __enum__: "com.watabou.mfcg.scenes.overlays.Position", toString: $estr }
	, TOP_RIGHT: { _hx_name: "TOP_RIGHT", _hx_index: 2, __enum__: "com.watabou.mfcg.scenes.overlays.Position", toString: $estr }
	, BOTTOM_LEFT: { _hx_name: "BOTTOM_LEFT", _hx_index: 3, __enum__: "com.watabou.mfcg.scenes.overlays.Position", toString: $estr }
	, BOTTOM_RIGHT: { _hx_name: "BOTTOM_RIGHT", _hx_index: 4, __enum__: "com.watabou.mfcg.scenes.overlays.Position", toString: $estr }
};
com_watabou_mfcg_scenes_overlays_Position.__constructs__ = [com_watabou_mfcg_scenes_overlays_Position.UNDEFINED, com_watabou_mfcg_scenes_overlays_Position.TOP_LEFT, com_watabou_mfcg_scenes_overlays_Position.TOP_RIGHT, com_watabou_mfcg_scenes_overlays_Position.BOTTOM_LEFT, com_watabou_mfcg_scenes_overlays_Position.BOTTOM_RIGHT];
var com_watabou_mfcg_scenes_overlays_EmblemOverlay = function (scene) {
	this.emblem = new com_watabou_mfcg_scenes_overlays_Emblem();
	this.emblem.ready.add($bind(this, this.layout));
	com_watabou_mfcg_scenes_overlays_Overlay.call(this, scene);
	this.addChild(this.emblem);
};
$hxClasses["com.watabou.mfcg.scenes.overlays.EmblemOverlay"] = com_watabou_mfcg_scenes_overlays_EmblemOverlay;
com_watabou_mfcg_scenes_overlays_EmblemOverlay.__name__ = "com.watabou.mfcg.scenes.overlays.EmblemOverlay";
com_watabou_mfcg_scenes_overlays_EmblemOverlay.__super__ = com_watabou_mfcg_scenes_overlays_Overlay;
com_watabou_mfcg_scenes_overlays_EmblemOverlay.prototype = $extend(com_watabou_mfcg_scenes_overlays_Overlay.prototype, {
	layout: function () {
		switch (com_watabou_mfcg_scenes_overlays_EmblemOverlay.pos._hx_index) {
			case 1:
				this.emblem.set_x(com_watabou_mfcg_scenes_overlays_Overlay.MARGIN);
				this.emblem.set_y(com_watabou_mfcg_scenes_overlays_Overlay.MARGIN);
				break;
			case 2:
				this.emblem.set_x(this.rWidth - this.emblem.get_width() - com_watabou_mfcg_scenes_overlays_Overlay.MARGIN);
				this.emblem.set_y(com_watabou_mfcg_scenes_overlays_Overlay.MARGIN);
				break;
			case 3:
				this.emblem.set_x(com_watabou_mfcg_scenes_overlays_Overlay.MARGIN);
				this.emblem.set_y(this.rHeight - this.emblem.get_height() - com_watabou_mfcg_scenes_overlays_Overlay.MARGIN);
				break;
			case 4:
				this.emblem.set_x(this.rWidth - this.emblem.get_width() - com_watabou_mfcg_scenes_overlays_Overlay.MARGIN);
				this.emblem.set_y(this.rHeight - this.emblem.get_height() - com_watabou_mfcg_scenes_overlays_Overlay.MARGIN);
				break;
			default:
		}
	}
	, get_position: function () {
		return com_watabou_mfcg_scenes_overlays_EmblemOverlay.pos;
	}
	, set_position: function (pos) {
		if (com_watabou_mfcg_scenes_overlays_EmblemOverlay.pos != pos) {
			com_watabou_mfcg_scenes_overlays_EmblemOverlay.pos = pos;
			this.layout();
		}
		return pos;
	}
	, onContext: function (context) {
		context.addItem("Customize...", ($_ = this.emblem, $bind($_, $_.customize)));
		context.addItem("Reroll", ($_ = this.emblem, $bind($_, $_.reroll)));
		context.addItem("Hide", function () {
			com_watabou_system_State.set("emblem", false);
		});
	}
	, exportPNG: function (state) {
		com_watabou_mfcg_scenes_overlays_Emblem.setResoltion(state);
	}
	, __class__: com_watabou_mfcg_scenes_overlays_EmblemOverlay
});
var com_watabou_mfcg_scenes_overlays_Frame = function () {
	this.padding = 10.0;
	com_watabou_coogee_ui_View.call(this);
};
$hxClasses["com.watabou.mfcg.scenes.overlays.Frame"] = com_watabou_mfcg_scenes_overlays_Frame;
com_watabou_mfcg_scenes_overlays_Frame.__name__ = "com.watabou.mfcg.scenes.overlays.Frame";
com_watabou_mfcg_scenes_overlays_Frame.__super__ = com_watabou_coogee_ui_View;
com_watabou_mfcg_scenes_overlays_Frame.prototype = $extend(com_watabou_coogee_ui_View.prototype, {
	layout: function () {
		var dark = com_watabou_mfcg_mapping_Style.colorDark;
		var g = this.get_graphics();
		g.clear();
		var unscaled = false;
		if (unscaled == null) {
			unscaled = true;
		}
		var gap = com_watabou_mfcg_mapping_Style.getStrokeWidth(com_watabou_mfcg_mapping_Style.strokeThick, unscaled) * 2;
		var unscaled = false;
		if (unscaled == null) {
			unscaled = true;
		}
		g.lineStyle(com_watabou_mfcg_mapping_Style.getStrokeWidth(com_watabou_mfcg_mapping_Style.strokeThick, unscaled), dark, null, null, null, null, 1);
		g.drawRect(0, 0, this.rWidth, this.rHeight);
		var unscaled = false;
		if (unscaled == null) {
			unscaled = true;
		}
		g.lineStyle(com_watabou_mfcg_mapping_Style.getStrokeWidth(com_watabou_mfcg_mapping_Style.strokeNormal, unscaled), dark, null, null, null, null, 1);
		g.drawRect(gap, gap, this.rWidth - gap * 2, this.rHeight - gap * 2);
	}
	, setInner: function (w, h) {
		this.setSize(Math.ceil(w + this.padding * 2), Math.ceil(h + this.padding * 2));
	}
	, outline: function (content) {
		this.setInner(content.get_width(), content.get_height());
		content.set_x(this.get_x() + this.padding);
		content.set_y(this.get_y() + this.padding);
	}
	, __class__: com_watabou_mfcg_scenes_overlays_Frame
});
var com_watabou_mfcg_scenes_overlays_Legend = function () {
	this.resized = new msignal_Signal0();
	com_watabou_coogee_ui_View.call(this);
	this.frame = new com_watabou_mfcg_scenes_overlays_Frame();
	this.add(this.frame);
	this.vbox = new com_watabou_coogee_ui_layouts_VBox();
	this.add(this.vbox);
	this.layout();
};
$hxClasses["com.watabou.mfcg.scenes.overlays.Legend"] = com_watabou_mfcg_scenes_overlays_Legend;
com_watabou_mfcg_scenes_overlays_Legend.__name__ = "com.watabou.mfcg.scenes.overlays.Legend";
com_watabou_mfcg_scenes_overlays_Legend.__super__ = com_watabou_coogee_ui_View;
com_watabou_mfcg_scenes_overlays_Legend.prototype = $extend(com_watabou_coogee_ui_View.prototype, {
	layout: function () {
		this.frame.outline(this.vbox);
		this.rWidth = this.frame.get_width();
		this.rHeight = this.frame.get_height();
		var g = this.get_graphics();
		g.clear();
		g.beginFill(com_watabou_mfcg_mapping_Style.colorPaper);
		g.drawRect(0, 0, this.rWidth, this.rHeight);
	}
	, relayout: function () {
		this.vbox.layout();
		this.layout();
		this.resized.dispatch();
	}
	, addView: function (view) {
		this.vbox.add(view);
	}
	, addItem: function (symb, desc) {
		var item = new com_watabou_mfcg_scenes_overlays__$Legend_LegendItem(this, symb, desc);
		this.addView(item);
		return item;
	}
	, addTitle: function (name) {
		var title = new com_watabou_mfcg_scenes_overlays__$Legend_TitleView(this, name);
		this.addView(title);
		return title;
	}
	, addEmblem: function () {
		var emblem = new com_watabou_mfcg_scenes_overlays__$Legend_EmblemView(this);
		this.addView(emblem);
		return emblem;
	}
	, addSeparator: function () {
		this.addView(new com_watabou_mfcg_scenes_overlays__$Legend_SeparatorView());
	}
	, addScale: function () {
		var scale = new com_watabou_mfcg_scenes_overlays__$Legend_ScaleBarView(this);
		scale.update(com_watabou_mfcg_scenes_TownScene.instance.map);
		this.addView(scale);
	}
	, wipe: function () {
		this.vbox.wipe();
		this.relayout();
	}
	, __class__: com_watabou_mfcg_scenes_overlays_Legend
});
var com_watabou_mfcg_scenes_overlays__$Legend_Legendary = function () { };
$hxClasses["com.watabou.mfcg.scenes.overlays._Legend.Legendary"] = com_watabou_mfcg_scenes_overlays__$Legend_Legendary;
com_watabou_mfcg_scenes_overlays__$Legend_Legendary.__name__ = "com.watabou.mfcg.scenes.overlays._Legend.Legendary";
com_watabou_mfcg_scenes_overlays__$Legend_Legendary.__isInterface__ = true;
var com_watabou_mfcg_scenes_overlays__$Legend_LegendItem = function (legend, symb, desc) {
	this.context = new msignal_Signal0();
	this.click = new msignal_Signal0();
	var _gthis = this;
	com_watabou_coogee_ui_View.call(this);
	this.legend = legend;
	var format = com_watabou_mfcg_ui_Text.getFormat("font_legend", com_watabou_mfcg_ui_Text.fontLegend, com_watabou_mfcg_mapping_Style.colorDark);
	var size = format.size;
	var col = size;
	var gap = size * 0.5;
	this.tfSymb = com_watabou_mfcg_ui_Text.get(symb + ".", format);
	this.tfSymb.set_x(col - this.tfSymb.get_width());
	this.tfSymb.mouseEnabled = false;
	this.addChild(this.tfSymb);
	this.tfDesc = com_watabou_mfcg_ui_Text.get(desc, format);
	this.tfDesc.set_x(col + gap);
	this.tfDesc.set_selectable(false);
	this.addChild(this.tfDesc);
	this.rWidth = col + gap + this.tfDesc.get_width();
	this.rHeight = this.tfDesc.get_height();
	this.tfDesc.addEventListener("mouseDown", function (e) {
		_gthis.click.dispatch();
	});
	this.tfDesc.addEventListener("rightClick", function (e) {
		_gthis.context.dispatch();
	});
};
$hxClasses["com.watabou.mfcg.scenes.overlays._Legend.LegendItem"] = com_watabou_mfcg_scenes_overlays__$Legend_LegendItem;
com_watabou_mfcg_scenes_overlays__$Legend_LegendItem.__name__ = "com.watabou.mfcg.scenes.overlays._Legend.LegendItem";
com_watabou_mfcg_scenes_overlays__$Legend_LegendItem.__interfaces__ = [com_watabou_mfcg_scenes_overlays__$Legend_Legendary];
com_watabou_mfcg_scenes_overlays__$Legend_LegendItem.__super__ = com_watabou_coogee_ui_View;
com_watabou_mfcg_scenes_overlays__$Legend_LegendItem.prototype = $extend(com_watabou_coogee_ui_View.prototype, {
	edit: function (update) {
		var _gthis = this;
		com_watabou_mfcg_ui_EditInPlace.fromTextField(this.tfDesc, this, 1, function (text) {
			if (text == "") {
				text = "-";
			}
			_gthis.tfDesc.set_text(text);
			var tmp = _gthis.tfDesc.get_x();
			var tmp1 = _gthis.tfDesc.get_width();
			_gthis.rWidth = tmp + tmp1;
			_gthis.legend.relayout();
			update(text);
		});
	}
	, __class__: com_watabou_mfcg_scenes_overlays__$Legend_LegendItem
});
var com_watabou_mfcg_scenes_overlays__$Legend_TitleView = function (legend, text) {
	this.context = new msignal_Signal0();
	this.click = new msignal_Signal0();
	var _gthis = this;
	com_watabou_coogee_ui_View.call(this);
	this.legend = legend;
	this.halign = "center";
	var format = com_watabou_mfcg_ui_Text.getFormat("font_title", com_watabou_mfcg_ui_Text.fontTitle, com_watabou_mfcg_mapping_Style.colorDark, 0.66666666666666663);
	this.tf = com_watabou_mfcg_ui_Text.get(text, format);
	this.tf.set_selectable(false);
	this.addChild(this.tf);
	this.rWidth = this.tf.get_width();
	this.rHeight = this.tf.get_height();
	this.tf.addEventListener("mouseDown", function (e) {
		_gthis.click.dispatch();
	});
	this.tf.addEventListener("rightClick", function (e) {
		_gthis.context.dispatch();
	});
};
$hxClasses["com.watabou.mfcg.scenes.overlays._Legend.TitleView"] = com_watabou_mfcg_scenes_overlays__$Legend_TitleView;
com_watabou_mfcg_scenes_overlays__$Legend_TitleView.__name__ = "com.watabou.mfcg.scenes.overlays._Legend.TitleView";
com_watabou_mfcg_scenes_overlays__$Legend_TitleView.__interfaces__ = [com_watabou_mfcg_scenes_overlays__$Legend_Legendary];
com_watabou_mfcg_scenes_overlays__$Legend_TitleView.__super__ = com_watabou_coogee_ui_View;
com_watabou_mfcg_scenes_overlays__$Legend_TitleView.prototype = $extend(com_watabou_coogee_ui_View.prototype, {
	edit: function (update) {
		var _gthis = this;
		com_watabou_mfcg_ui_EditInPlace.fromTextField(this.tf, this, 0, function (text) {
			if (text == "") {
				text = "-";
			}
			_gthis.tf.set_text(text);
			_gthis.rWidth = _gthis.tf.get_width();
			_gthis.legend.relayout();
			if (update != null) {
				update(text);
			}
		});
	}
	, __class__: com_watabou_mfcg_scenes_overlays__$Legend_TitleView
});
var com_watabou_mfcg_scenes_overlays__$Legend_SeparatorView = function () {
	com_watabou_coogee_ui_View.call(this);
	this.halign = "fill";
	var unscaled = false;
	if (unscaled == null) {
		unscaled = true;
	}
	this.set_height(com_watabou_mfcg_mapping_Style.getStrokeWidth(com_watabou_mfcg_mapping_Style.strokeNormal, unscaled));
};
$hxClasses["com.watabou.mfcg.scenes.overlays._Legend.SeparatorView"] = com_watabou_mfcg_scenes_overlays__$Legend_SeparatorView;
com_watabou_mfcg_scenes_overlays__$Legend_SeparatorView.__name__ = "com.watabou.mfcg.scenes.overlays._Legend.SeparatorView";
com_watabou_mfcg_scenes_overlays__$Legend_SeparatorView.__interfaces__ = [com_watabou_mfcg_scenes_overlays__$Legend_Legendary];
com_watabou_mfcg_scenes_overlays__$Legend_SeparatorView.__super__ = com_watabou_coogee_ui_View;
com_watabou_mfcg_scenes_overlays__$Legend_SeparatorView.prototype = $extend(com_watabou_coogee_ui_View.prototype, {
	layout: function () {
		this.get_graphics().clear();
		this.get_graphics().beginFill(com_watabou_mfcg_mapping_Style.colorDark);
		this.get_graphics().drawRect(0, 0, this.rWidth, this.rHeight);
	}
	, __class__: com_watabou_mfcg_scenes_overlays__$Legend_SeparatorView
});
var com_watabou_mfcg_scenes_overlays__$Legend_ScaleBarView = function (legend) {
	com_watabou_coogee_ui_View.call(this);
	this.legend = legend;
	this.halign = "center";
	this.scalebar = com_watabou_mfcg_scenes_overlays_ScaleBar.create(true);
	this.addChild(this.scalebar);
	this.addEventListener("click", $bind(this, this.onClick));
	this.addEventListener("rightClick", $bind(this, this.onContext));
};
$hxClasses["com.watabou.mfcg.scenes.overlays._Legend.ScaleBarView"] = com_watabou_mfcg_scenes_overlays__$Legend_ScaleBarView;
com_watabou_mfcg_scenes_overlays__$Legend_ScaleBarView.__name__ = "com.watabou.mfcg.scenes.overlays._Legend.ScaleBarView";
com_watabou_mfcg_scenes_overlays__$Legend_ScaleBarView.__interfaces__ = [com_watabou_mfcg_scenes_overlays__$Legend_Legendary];
com_watabou_mfcg_scenes_overlays__$Legend_ScaleBarView.__super__ = com_watabou_coogee_ui_View;
com_watabou_mfcg_scenes_overlays__$Legend_ScaleBarView.prototype = $extend(com_watabou_coogee_ui_View.prototype, {
	update: function (map) {
		this.map = map;
		this.scalebar.update(map);
		this.rWidth = this.scalebar.get_width();
		this.rHeight = this.scalebar.get_height() - 4;
		this.scalebar.set_y(this.rHeight);
	}
	, onClick: function (e) {
		if (e.shiftKey) {
			this.removeChild(this.scalebar);
			com_watabou_mfcg_scenes_overlays_ScaleBar.toggleView();
			this.scalebar = com_watabou_mfcg_scenes_overlays_ScaleBar.create(true);
			this.addChild(this.scalebar);
		} else {
			com_watabou_mfcg_scenes_overlays_ScaleBar.toggleSystem();
		}
		this.update(this.map);
		this.legend.relayout();
	}
	, onContext: function (e) {
		var _gthis = this;
		var context = com_watabou_mfcg_scenes_TownScene.context;
		var addSystem = function (name, system) {
			context.addItem(name, function () {
				com_watabou_mfcg_scenes_overlays_ScaleBar.system = system;
				_gthis.scalebar.update();
				_gthis.update(_gthis.map);
				_gthis.legend.relayout();
			}, com_watabou_mfcg_scenes_overlays_ScaleBar.system == system);
		};
		addSystem("Metric units", com_watabou_mfcg_scenes_overlays_ScaleBar.METRIC);
		addSystem("Imperial units", com_watabou_mfcg_scenes_overlays_ScaleBar.IMPERIAL);
		var addView = function (name, view) {
			context.addItem(name, function () {
				com_watabou_mfcg_scenes_overlays_ScaleBar.sbClass = view;
				_gthis.removeChild(_gthis.scalebar);
				_gthis.scalebar = com_watabou_mfcg_scenes_overlays_ScaleBar.create(true);
				_gthis.addChild(_gthis.scalebar);
				_gthis.update(_gthis.map);
				_gthis.legend.relayout();
			}, com_watabou_mfcg_scenes_overlays_ScaleBar.sbClass == view);
		};
		addView("Default style", com_watabou_mfcg_scenes_overlays_ScaleBarOld);
		addView("Alternative style", com_watabou_mfcg_scenes_overlays_ScaleBarNew);
		context.addItem("Hide", function () {
			com_watabou_system_State.set("scale_bar", false);
		});
	}
	, __class__: com_watabou_mfcg_scenes_overlays__$Legend_ScaleBarView
});
var com_watabou_mfcg_scenes_overlays__$Legend_EmblemView = function (legend) {
	com_watabou_coogee_ui_View.call(this);
	this.legend = legend;
	this.halign = "center";
	this.emblem = new com_watabou_mfcg_scenes_overlays_Emblem();
	this.emblem.ready.add($bind(this, this.layout));
	this.rWidth = this.emblem.get_width();
	this.rHeight = this.emblem.get_height();
	this.addChild(this.emblem);
	this.addEventListener("rightClick", $bind(this, this.onContext));
};
$hxClasses["com.watabou.mfcg.scenes.overlays._Legend.EmblemView"] = com_watabou_mfcg_scenes_overlays__$Legend_EmblemView;
com_watabou_mfcg_scenes_overlays__$Legend_EmblemView.__name__ = "com.watabou.mfcg.scenes.overlays._Legend.EmblemView";
com_watabou_mfcg_scenes_overlays__$Legend_EmblemView.__interfaces__ = [com_watabou_mfcg_scenes_overlays__$Legend_Legendary];
com_watabou_mfcg_scenes_overlays__$Legend_EmblemView.__super__ = com_watabou_coogee_ui_View;
com_watabou_mfcg_scenes_overlays__$Legend_EmblemView.prototype = $extend(com_watabou_coogee_ui_View.prototype, {
	layout: function () {
		this.rWidth = this.emblem.get_width();
		this.rHeight = this.emblem.get_height();
		this.legend.relayout();
	}
	, onContext: function (e) {
		var context = com_watabou_mfcg_scenes_TownScene.context;
		context.addItem("Customize...", ($_ = this.emblem, $bind($_, $_.customize)));
		context.addItem("Reroll", ($_ = this.emblem, $bind($_, $_.reroll)));
		context.addItem("Hide", function () {
			com_watabou_system_State.set("emblem", false);
		});
	}
	, __class__: com_watabou_mfcg_scenes_overlays__$Legend_EmblemView
});
var com_watabou_mfcg_scenes_overlays_LegendOverlay = function (scene) {
	this.legend = new com_watabou_mfcg_scenes_overlays_Legend();
	this.legend.resized.add($bind(this, this.layout));
	com_watabou_mfcg_scenes_overlays_Overlay.call(this, scene);
	this.addChild(this.legend);
	com_watabou_mfcg_model_ModelDispatcher.titleChanged.add($bind(this, this.onChangedStr));
	com_watabou_mfcg_model_ModelDispatcher.districtsChanged.add($bind(this, this.onChangedVoid));
	com_watabou_mfcg_model_ModelDispatcher.landmarksChanged.add($bind(this, this.onChangedVoid));
};
$hxClasses["com.watabou.mfcg.scenes.overlays.LegendOverlay"] = com_watabou_mfcg_scenes_overlays_LegendOverlay;
com_watabou_mfcg_scenes_overlays_LegendOverlay.__name__ = "com.watabou.mfcg.scenes.overlays.LegendOverlay";
com_watabou_mfcg_scenes_overlays_LegendOverlay.__super__ = com_watabou_mfcg_scenes_overlays_Overlay;
com_watabou_mfcg_scenes_overlays_LegendOverlay.prototype = $extend(com_watabou_mfcg_scenes_overlays_Overlay.prototype, {
	layout: function () {
		if (com_watabou_mfcg_scenes_overlays_LegendOverlay.auto) {
			com_watabou_mfcg_scenes_overlays_LegendOverlay.pos = this.getAutoPos();
		}
		var p = this.pos2point(com_watabou_mfcg_scenes_overlays_LegendOverlay.pos);
		this.legend.set_x(p.x);
		this.legend.set_y(p.y);
		this.scene.arrangeOverlays();
	}
	, get_position: function () {
		return com_watabou_mfcg_scenes_overlays_LegendOverlay.pos;
	}
	, set_position: function (pos) {
		com_watabou_mfcg_scenes_overlays_LegendOverlay.auto = false;
		if (com_watabou_mfcg_scenes_overlays_LegendOverlay.pos != pos) {
			com_watabou_mfcg_scenes_overlays_LegendOverlay.pos = pos;
			this.layout();
		}
		return pos;
	}
	, getAutoPos: function () {
		var _gthis = this;
		var map = this.scene.map;
		if (map == null) {
			return com_watabou_mfcg_scenes_overlays_Position.TOP_LEFT;
		}
		return com_watabou_utils_ArrayExtender.min([com_watabou_mfcg_scenes_overlays_Position.TOP_LEFT, com_watabou_mfcg_scenes_overlays_Position.BOTTOM_LEFT, com_watabou_mfcg_scenes_overlays_Position.BOTTOM_RIGHT, com_watabou_mfcg_scenes_overlays_Position.TOP_RIGHT], function (pos) {
			var p1 = _gthis.pos2point(pos);
			var p2 = new openfl_geom_Point(p1.x + _gthis.legend.get_width(), p1.y + _gthis.legend.get_height());
			var pp1 = _gthis.layer2map(map, p1);
			var pp2 = _gthis.layer2map(map, p2);
			var rect = new openfl_geom_Rectangle(pp1.x, pp1.y, pp2.x - pp1.x, pp2.y - pp1.y);
			return _gthis.model.getDetails(rect);
		});
	}
	, pos2point: function (pos) {
		switch (pos._hx_index) {
			case 1:
				return new openfl_geom_Point(com_watabou_mfcg_scenes_overlays_Overlay.MARGIN, com_watabou_mfcg_scenes_overlays_Overlay.MARGIN);
			case 2:
				return new openfl_geom_Point(this.rWidth - this.legend.get_width() - com_watabou_mfcg_scenes_overlays_Overlay.MARGIN, com_watabou_mfcg_scenes_overlays_Overlay.MARGIN);
			case 3:
				return new openfl_geom_Point(com_watabou_mfcg_scenes_overlays_Overlay.MARGIN, this.rHeight - this.legend.get_height() - com_watabou_mfcg_scenes_overlays_Overlay.MARGIN);
			case 4:
				return new openfl_geom_Point(this.rWidth - this.legend.get_width() - com_watabou_mfcg_scenes_overlays_Overlay.MARGIN, this.rHeight - this.legend.get_height() - com_watabou_mfcg_scenes_overlays_Overlay.MARGIN);
			default:
				return null;
		}
	}
	, update: function (model) {
		com_watabou_mfcg_scenes_overlays_Overlay.prototype.update.call(this, model);
		if (!this.get_visible()) {
			return;
		}
		this.legend.wipe();
		var context = com_watabou_mfcg_scenes_TownScene.context;
		var title = com_watabou_system_State.get("city_name", 1);
		var scale = com_watabou_system_State.get("scale_bar", true);
		var emblem = com_watabou_system_State.get("emblem", false);
		var districts = com_watabou_system_State.get("districts", "Curved") == "Legend" && model.districts.length > 0;
		var landmarks = com_watabou_system_State.get("landmarks") == "Legend" && model.landmarks.length > 0;
		if (title) {
			var title = this.legend.addTitle(model.name);
			var edit = function () {
				title.edit(function (name) {
					model.setName(name);
				});
			};
			title.click.add(edit);
			title.context.add(function () {
				context.addItem("Edit name", edit);
				context.addItem("Reroll", function () {
					model.setName(model.rerollName());
				});
			});
			if ((districts || landmarks) && !scale && !emblem) {
				this.legend.addSeparator();
			}
		}
		if (emblem) {
			this.legend.addEmblem();
		}
		if (scale) {
			this.legend.addScale();
		}
		if (districts) {
			var _g = 0;
			var _g1 = model.districts.length;
			while (_g < _g1) {
				var i = _g++;
				var district = [model.districts[i]];
				var sym = com_watabou_mfcg_scenes_overlays_Pin.number(i);
				var update = [(function (district) {
					return function (name) {
						return district[0].name = name;
					};
				})(district)];
				var item = [this.legend.addItem(sym, district[0].name)];
				var edit1 = [(function (item, update) {
					return function () {
						item[0].edit(update[0]);
					};
				})(item, update)];
				item[0].click.add(edit1[0]);
				item[0].context.add((function (edit) {
					return function () {
						context.addItem("Edit name", edit[0]);
						context.addItem("Reroll all", $bind(model, model.rerollDistricts));
					};
				})(edit1));
			}
			if (landmarks) {
				this.legend.addSeparator();
			}
		}
		if (landmarks) {
			var _g = 0;
			var _g1 = model.landmarks.length;
			while (_g < _g1) {
				var i = _g++;
				var landmark = [model.landmarks[i]];
				var sym = com_watabou_mfcg_scenes_overlays_Marker.letter(i);
				var updateName = [(function (landmark) {
					return function (name) {
						return landmark[0].name = name;
					};
				})(landmark)];
				var item1 = [this.legend.addItem(sym, landmark[0].name)];
				var edit2 = [(function (item, updateName) {
					return function () {
						item[0].edit(updateName[0]);
					};
				})(item1, updateName)];
				item1[0].click.add(edit2[0]);
				item1[0].context.add((function (edit, landmark) {
					return function () {
						context.addItem("Edit name", edit[0]);
						context.addItem("Delete", (function (landmark) {
							return function () {
								model.removeLandmark(landmark[0]);
							};
						})(landmark));
					};
				})(edit2, landmark));
			}
		}
		this.legend.layout();
		this.layout();
	}
	, onContext: function (context) {
		var _gthis = this;
		var sub = new com_watabou_coogee_ui_Menu();
		var value = com_watabou_mfcg_scenes_overlays_Position.TOP_LEFT;
		sub.addItem("Top-left", function () {
			_gthis.set_position(value);
		}, _gthis.get_position() == value && !com_watabou_mfcg_scenes_overlays_LegendOverlay.auto);
		var value1 = com_watabou_mfcg_scenes_overlays_Position.TOP_RIGHT;
		sub.addItem("Top-right", function () {
			_gthis.set_position(value1);
		}, _gthis.get_position() == value1 && !com_watabou_mfcg_scenes_overlays_LegendOverlay.auto);
		var value2 = com_watabou_mfcg_scenes_overlays_Position.BOTTOM_LEFT;
		sub.addItem("Bottom-left", function () {
			_gthis.set_position(value2);
		}, _gthis.get_position() == value2 && !com_watabou_mfcg_scenes_overlays_LegendOverlay.auto);
		var value3 = com_watabou_mfcg_scenes_overlays_Position.BOTTOM_RIGHT;
		sub.addItem("Bottom-right", function () {
			_gthis.set_position(value3);
		}, _gthis.get_position() == value3 && !com_watabou_mfcg_scenes_overlays_LegendOverlay.auto);
		sub.addItem("Auto", function () {
			com_watabou_mfcg_scenes_overlays_LegendOverlay.auto = true;
			_gthis.layout();
		}, com_watabou_mfcg_scenes_overlays_LegendOverlay.auto);
		context.group();
		context.addSubmenu("Position", sub);
		context.addItem("Hide", function () {
			if (com_watabou_system_State.get("landmarks") == "Legend") {
				com_watabou_system_State.set("landmarks", "Icon");
			}
			if (com_watabou_system_State.get("districts", "Curved") == "Legend") {
				com_watabou_system_State.set("districts", "Curved");
			} else {
				com_watabou_mfcg_scenes_TownScene.instance.toggleOverlays();
			}
		});
	}
	, onChangedStr: function (str) {
		this.update(this.model);
	}
	, onChangedVoid: function () {
		this.update(this.model);
	}
	, onDestroy: function () {
		com_watabou_mfcg_scenes_overlays_Overlay.prototype.onDestroy.call(this);
		com_watabou_mfcg_model_ModelDispatcher.titleChanged.remove($bind(this, this.onChangedStr));
		com_watabou_mfcg_model_ModelDispatcher.districtsChanged.remove($bind(this, this.onChangedVoid));
	}
	, __class__: com_watabou_mfcg_scenes_overlays_LegendOverlay
});
var com_watabou_mfcg_scenes_overlays_Marker = function (overlay) {
	this.moved = false;
	openfl_display_Sprite.call(this);
	this.overlay = overlay;
	this.draw();
	this.set_buttonMode(true);
	this.addEventListener("click", $bind(this, this.onClick));
	this.addEventListener("rightClick", $bind(this, this.onContext));
	this.addEventListener("rollOver", $bind(this, this.onRollOver));
	this.addEventListener("mouseDown", $bind(this, this.onMouseDown));
};
$hxClasses["com.watabou.mfcg.scenes.overlays.Marker"] = com_watabou_mfcg_scenes_overlays_Marker;
com_watabou_mfcg_scenes_overlays_Marker.__name__ = "com.watabou.mfcg.scenes.overlays.Marker";
com_watabou_mfcg_scenes_overlays_Marker.letter = function (i) {
	var code = HxOverrides.cca("a", 0) + i;
	return String.fromCodePoint(code);
};
com_watabou_mfcg_scenes_overlays_Marker.__super__ = openfl_display_Sprite;
com_watabou_mfcg_scenes_overlays_Marker.prototype = $extend(openfl_display_Sprite.prototype, {
	set: function (landmark, sym) {
		this.landmark = landmark;
		this.set_symbol(sym);
	}
	, draw: function () {
		var format = com_watabou_mfcg_ui_Text.getFormat("font_pin", com_watabou_mfcg_ui_Text.fontPin, com_watabou_mfcg_mapping_Style.colorDark, 1.14285714285714279);
		var size = format.size;
		var r = size * 0.66666666666666663;
		var a = 0.166666666666666657 * Math.PI;
		var cos = Math.cos(a);
		var sin = Math.sin(a);
		var h = r / sin;
		var g = this.get_graphics();
		var unscaled = false;
		if (unscaled == null) {
			unscaled = true;
		}
		g.lineStyle(com_watabou_mfcg_mapping_Style.getStrokeWidth(com_watabou_mfcg_mapping_Style.strokeNormal, unscaled) * 2, com_watabou_mfcg_mapping_Style.colorPaper);
		g.drawCircle(0, -h, r);
		g.moveTo(-r * cos, -h + r * sin);
		g.lineTo(0, 0);
		g.lineTo(r * cos, -h + r * sin);
		g.endFill();
		g.beginFill(com_watabou_mfcg_mapping_Style.colorDark);
		g.drawCircle(0, -h, r);
		g.beginFill(com_watabou_mfcg_mapping_Style.colorDark);
		g.moveTo(-r * cos, -h + r * sin);
		g.lineTo(0, 0);
		g.lineTo(r * cos, -h + r * sin);
		g.beginFill(com_watabou_mfcg_mapping_Style.colorPaper);
		var tmp = this.get_graphics();
		var unscaled = false;
		if (unscaled == null) {
			unscaled = true;
		}
		tmp.drawCircle(0, -h, r - com_watabou_mfcg_mapping_Style.getStrokeWidth(com_watabou_mfcg_mapping_Style.strokeThick, unscaled));
		this.tf = com_watabou_mfcg_ui_Text.get("a", format);
		this.tf.mouseEnabled = false;
		this.tf.set_y(-h - this.tf.get_height() / 2);
		this.addChild(this.tf);
	}
	, get_symbol: function () {
		return this.tf.get_text();
	}
	, set_symbol: function (value) {
		this.tf.set_text(value);
		this.tf.set_x(-this.tf.get_width() / 2);
		return value;
	}
	, onClick: function (e) {
		if (!this.moved) {
			this.overlay.editMarker(this.landmark);
			e.stopPropagation();
		}
	}
	, onRollOver: function (e) {
		com_watabou_mfcg_ui_Tooltip.instance.set(this.landmark.name);
	}
	, onMouseDown: function (e) {
		this.stage.addEventListener("mouseMove", $bind(this, this.onMouseMove));
		this.stage.addEventListener("mouseUp", $bind(this, this.onMouseUp));
		this.grabX = this.get_mouseX();
		this.grabY = this.get_mouseY();
		this.moved = false;
	}
	, onMouseMove: function (e) {
		this.moved = true;
		this.set_x(this.parent.get_mouseX() - this.grabX);
		this.set_y(this.parent.get_mouseY() - this.grabY);
		e.updateAfterEvent();
	}
	, onMouseUp: function (e) {
		this.stage.removeEventListener("mouseMove", $bind(this, this.onMouseMove));
		this.stage.removeEventListener("mouseUp", $bind(this, this.onMouseUp));
		this.overlay.onDrag(this);
	}
	, onContext: function (e) {
		this.overlay.landmark = this.landmark;
	}
	, __class__: com_watabou_mfcg_scenes_overlays_Marker
	, __properties__: $extend(openfl_display_Sprite.prototype.__properties__, { set_symbol: "set_symbol", get_symbol: "get_symbol" })
});
var com_watabou_mfcg_scenes_overlays_MarkersOverlay = function (scene) {
	var _gthis = this;
	com_watabou_mfcg_scenes_overlays_Overlay.call(this, scene);
	com_watabou_mfcg_model_ModelDispatcher.landmarksChanged.add(function () {
		_gthis.update(_gthis.model);
	});
};
$hxClasses["com.watabou.mfcg.scenes.overlays.MarkersOverlay"] = com_watabou_mfcg_scenes_overlays_MarkersOverlay;
com_watabou_mfcg_scenes_overlays_MarkersOverlay.__name__ = "com.watabou.mfcg.scenes.overlays.MarkersOverlay";
com_watabou_mfcg_scenes_overlays_MarkersOverlay.__super__ = com_watabou_mfcg_scenes_overlays_Overlay;
com_watabou_mfcg_scenes_overlays_MarkersOverlay.prototype = $extend(com_watabou_mfcg_scenes_overlays_Overlay.prototype, {
	layout: function () {
		com_watabou_mfcg_scenes_overlays_Overlay.prototype.layout.call(this);
		this.sync();
	}
	, removeAll: function () {
		this.removeChildren();
	}
	, sync: function () {
		var _g = 0;
		var _g1 = this.get_numChildren();
		while (_g < _g1) {
			var i = _g++;
			var marker = this.getChildAt(i);
			var p = this.map2layer(this.scene.map, marker.landmark.pos);
			marker.set_x(p.x);
			marker.set_y(p.y);
		}
	}
	, update: function (model) {
		com_watabou_mfcg_scenes_overlays_Overlay.prototype.update.call(this, model);
		var _g = 0;
		var _g1 = model.landmarks.length;
		while (_g < _g1) {
			var i = _g++;
			var lm = model.landmarks[i];
			var m = this.getChildAt(i);
			if (m == null) {
				m = new com_watabou_mfcg_scenes_overlays_Marker(this);
				this.addChild(m);
			}
			m.set(lm, com_watabou_mfcg_scenes_overlays_Marker.letter(i));
			if (this.scene.map != null) {
				var p = this.map2layer(this.scene.map, lm.pos);
				m.set_x(p.x);
				m.set_y(p.y);
			}
		}
		while (this.get_numChildren() > model.landmarks.length) this.removeChildAt(model.landmarks.length);
	}
	, editMarker: function (landmark) {
		com_watabou_coogee_ui_UI.showDialog(new com_watabou_mfcg_ui_forms_MarkerForm(landmark, false));
	}
	, onDrag: function (m) {
		var tmp = this.scene.map;
		var tmp1 = m.get_x();
		var tmp2 = m.get_y();
		m.landmark.pos = this.layer2map(tmp, new openfl_geom_Point(tmp1, tmp2));
		m.landmark.assign();
	}
	, reorder: function () {
		var _g = 0;
		var _g1 = this.get_numChildren();
		while (_g < _g1) {
			var i = _g++;
			var marker = this.getChildAt(i);
			marker.set_symbol(com_watabou_mfcg_scenes_overlays_Marker.letter(i));
		}
	}
	, onContext: function (context) {
		var _gthis = this;
		context.addItem("Edit", function () {
			_gthis.editMarker(_gthis.landmark);
		});
		context.addItem("Hide", function () {
			com_watabou_system_State.set("landmarks", "Hidden");
		});
	}
	, __class__: com_watabou_mfcg_scenes_overlays_MarkersOverlay
});
var com_watabou_mfcg_scenes_overlays_PinsOverlay = function (scene) {
	com_watabou_mfcg_scenes_overlays_Overlay.call(this, scene);
	this.mouseChildren = false;
};
$hxClasses["com.watabou.mfcg.scenes.overlays.PinsOverlay"] = com_watabou_mfcg_scenes_overlays_PinsOverlay;
com_watabou_mfcg_scenes_overlays_PinsOverlay.__name__ = "com.watabou.mfcg.scenes.overlays.PinsOverlay";
com_watabou_mfcg_scenes_overlays_PinsOverlay.__super__ = com_watabou_mfcg_scenes_overlays_Overlay;
com_watabou_mfcg_scenes_overlays_PinsOverlay.prototype = $extend(com_watabou_mfcg_scenes_overlays_Overlay.prototype, {
	layout: function () {
		com_watabou_mfcg_scenes_overlays_Overlay.prototype.layout.call(this);
		this.sync();
	}
	, sync: function () {
		var _g = 0;
		var _g1 = this.get_numChildren();
		while (_g < _g1) {
			var i = _g++;
			var pin = this.getChildAt(i);
			var p = this.map2layer(this.scene.map, pin.pos);
			pin.set_x(p.x);
			pin.set_y(p.y);
		}
	}
	, update: function (model) {
		var _g = 0;
		var _g1 = model.districts.length;
		while (_g < _g1) {
			var i = _g++;
			var district = model.districts[i];
			var sym = com_watabou_mfcg_scenes_overlays_Pin.number(i);
			this.addChild(new com_watabou_mfcg_scenes_overlays_Pin(sym, district.label.pos));
		}
	}
	, __class__: com_watabou_mfcg_scenes_overlays_PinsOverlay
});
var com_watabou_mfcg_scenes_overlays_Pin = function (sym, pos) {
	openfl_display_Sprite.call(this);
	this.pos = pos;
	var format = com_watabou_mfcg_ui_Text.getFormat("font_pin", com_watabou_mfcg_ui_Text.fontPin, com_watabou_mfcg_mapping_Style.colorPaper);
	this.tf = com_watabou_mfcg_ui_Text.get(sym, format);
	this.tf.set_x(-this.tf.get_width() / 2);
	this.tf.set_y(-this.tf.get_height() / 2);
	this.addChild(this.tf);
	var size = format.size;
	this.get_graphics().beginFill(com_watabou_mfcg_mapping_Style.colorPaper);
	this.get_graphics().drawCircle(0, 0, size);
	this.get_graphics().beginFill(com_watabou_mfcg_mapping_Style.colorDark);
	var tmp = this.get_graphics();
	var unscaled = false;
	if (unscaled == null) {
		unscaled = true;
	}
	tmp.drawCircle(0, 0, size - com_watabou_mfcg_mapping_Style.getStrokeWidth(com_watabou_mfcg_mapping_Style.strokeThick, unscaled));
};
$hxClasses["com.watabou.mfcg.scenes.overlays.Pin"] = com_watabou_mfcg_scenes_overlays_Pin;
com_watabou_mfcg_scenes_overlays_Pin.__name__ = "com.watabou.mfcg.scenes.overlays.Pin";
com_watabou_mfcg_scenes_overlays_Pin.number = function (i) {
	return Std.string(i + 1);
};
com_watabou_mfcg_scenes_overlays_Pin.__super__ = openfl_display_Sprite;
com_watabou_mfcg_scenes_overlays_Pin.prototype = $extend(openfl_display_Sprite.prototype, {
	__class__: com_watabou_mfcg_scenes_overlays_Pin
});
var com_watabou_mfcg_scenes_overlays_ScaleBar = function () {
	this.scale = 0.0;
	openfl_display_Sprite.call(this);
	this.format = com_watabou_mfcg_ui_Text.getFormat("font_element", com_watabou_mfcg_ui_Text.fontElement, com_watabou_mfcg_mapping_Style.colorDark);
};
$hxClasses["com.watabou.mfcg.scenes.overlays.ScaleBar"] = com_watabou_mfcg_scenes_overlays_ScaleBar;
com_watabou_mfcg_scenes_overlays_ScaleBar.__name__ = "com.watabou.mfcg.scenes.overlays.ScaleBar";
com_watabou_mfcg_scenes_overlays_ScaleBar.toggleSystem = function () {
	com_watabou_mfcg_scenes_overlays_ScaleBar.system = com_watabou_mfcg_scenes_overlays_ScaleBar.system == com_watabou_mfcg_scenes_overlays_ScaleBar.METRIC ? com_watabou_mfcg_scenes_overlays_ScaleBar.IMPERIAL : com_watabou_mfcg_scenes_overlays_ScaleBar.METRIC;
};
com_watabou_mfcg_scenes_overlays_ScaleBar.toggleView = function () {
	com_watabou_mfcg_scenes_overlays_ScaleBar.sbClass = com_watabou_mfcg_scenes_overlays_ScaleBar.sbClass != com_watabou_mfcg_scenes_overlays_ScaleBarOld ? com_watabou_mfcg_scenes_overlays_ScaleBarOld : com_watabou_mfcg_scenes_overlays_ScaleBarNew;
};
com_watabou_mfcg_scenes_overlays_ScaleBar.create = function (embeded) {
	if (embeded == null) {
		embeded = false;
	}
	if (com_watabou_mfcg_scenes_overlays_ScaleBar.sbClass == null) {
		com_watabou_mfcg_scenes_overlays_ScaleBar.sbClass = com_watabou_mfcg_scenes_overlays_ScaleBarOld;
	}
	return Type.createInstance(com_watabou_mfcg_scenes_overlays_ScaleBar.sbClass, [embeded]);
};
com_watabou_mfcg_scenes_overlays_ScaleBar.__super__ = openfl_display_Sprite;
com_watabou_mfcg_scenes_overlays_ScaleBar.prototype = $extend(openfl_display_Sprite.prototype, {
	update: function (map) {
		this.get_graphics().clear();
		var minSize = this.getMinSize();
		if (map != null) {
			this.scale = this.getScale(map);
		}
		com_watabou_mfcg_scenes_overlays_ScaleBar.units = com_watabou_mfcg_scenes_overlays_ScaleBar.system;
		while (true) {
			var px2unit = com_watabou_mfcg_scenes_overlays_ScaleBar.units.px2unit * this.scale;
			this.tickUnit = Math.pow(10, Math.ceil(Math.log(minSize / px2unit) / Math.log(10)));
			this.tickPx = this.tickUnit * px2unit;
			if (this.tickPx > 5 * minSize) {
				this.tickUnit /= 5;
				this.tickPx /= 5;
			} else if (this.tickPx > 4 * minSize) {
				this.tickUnit /= 4;
				this.tickPx /= 4;
			} else if (this.tickPx > 2 * minSize) {
				this.tickUnit /= 2;
				this.tickPx /= 2;
			}
			if (this.tickUnit <= 1 && com_watabou_mfcg_scenes_overlays_ScaleBar.units.sub != null) {
				com_watabou_mfcg_scenes_overlays_ScaleBar.units = com_watabou_mfcg_scenes_overlays_ScaleBar.units.sub;
			} else {
				break;
			}
		}
	}
	, getMinSize: function () {
		return 100;
	}
	, createLabel: function (txt) {
		if (txt == null) {
			txt = "";
		}
		var tf = com_watabou_mfcg_ui_Text.get(txt, this.format);
		tf.set_selectable(false);
		return tf;
	}
	, getScale: function (map) {
		var scale = 1.0;
		var p = this;
		while (p != null) {
			scale /= p.get_scaleX();
			p = p.parent;
		}
		p = map;
		while (p != null) {
			scale *= p.get_scaleX();
			p = p.parent;
		}
		return scale;
	}
	, __class__: com_watabou_mfcg_scenes_overlays_ScaleBar
});
var com_watabou_mfcg_scenes_overlays_ScaleBarNew = function (embeded) {
	if (embeded == null) {
		embeded = false;
	}
	this.embeded = embeded;
	com_watabou_mfcg_scenes_overlays_ScaleBar.call(this);
	this.black = com_watabou_mfcg_mapping_Style.colorDark;
	this.white = com_watabou_mfcg_mapping_Style.colorPaper;
	this.grey = com_watabou_mfcg_mapping_Style.colorDark;
	this.tfValues = [];
	var tf = this.createLabel();
	this.tfValues.push(tf);
	if (!embeded || true) {
		this.addChild(tf);
	}
	var tf = this.createLabel();
	this.tfValues.push(tf);
	if (!embeded || true) {
		this.addChild(tf);
	}
	var tf = this.createLabel();
	this.tfValues.push(tf);
	if (!embeded || true) {
		this.addChild(tf);
	}
	var tf = this.createLabel();
	this.tfValues.push(tf);
	if (!embeded || true) {
		this.addChild(tf);
	}
	var tf = this.createLabel();
	this.tfValues.push(tf);
	if (!embeded) {
		this.addChild(tf);
	}
	this.tfUnits = this.createLabel();
	this.addChild(this.tfUnits);
};
$hxClasses["com.watabou.mfcg.scenes.overlays.ScaleBarNew"] = com_watabou_mfcg_scenes_overlays_ScaleBarNew;
com_watabou_mfcg_scenes_overlays_ScaleBarNew.__name__ = "com.watabou.mfcg.scenes.overlays.ScaleBarNew";
com_watabou_mfcg_scenes_overlays_ScaleBarNew.__super__ = com_watabou_mfcg_scenes_overlays_ScaleBar;
com_watabou_mfcg_scenes_overlays_ScaleBarNew.prototype = $extend(com_watabou_mfcg_scenes_overlays_ScaleBar.prototype, {
	update: function (map) {
		com_watabou_mfcg_scenes_overlays_ScaleBar.prototype.update.call(this, map);
		var w = (this.tickPx - 2) / 5;
		var _g = 0;
		while (_g < 5) {
			var i = _g++;
			var tf = this.tfValues[i];
			tf.set_text(Std.string(this.tickUnit * (i + 1) / 5));
			tf.set_x(this.tickPx * (i + 1) / 5 - tf.get_width() / 2);
			tf.set_y(-12. - tf.get_height() + 2);
			var tmp = (i & 1) == 0 ? this.white : this.grey;
			this.get_graphics().beginFill(tmp);
			this.get_graphics().drawRect(w * i + 1, -8, w, 8);
		}
		var tmp = this.get_graphics();
		var unscaled = false;
		if (unscaled == null) {
			unscaled = true;
		}
		tmp.lineStyle(com_watabou_mfcg_mapping_Style.getStrokeWidth(com_watabou_mfcg_mapping_Style.strokeNormal, unscaled), com_watabou_mfcg_mapping_Style.colorDark, 1, true, null, 0, 1);
		this.get_graphics().moveTo(0, 0);
		this.get_graphics().lineTo(this.tickPx, 0);
		this.get_graphics().moveTo(0, -8);
		this.get_graphics().lineTo(this.tickPx, -8);
		this.get_graphics().moveTo(0, -12.);
		this.get_graphics().lineTo(0, 0.);
		this.get_graphics().moveTo(this.tickPx, -12.);
		this.get_graphics().lineTo(this.tickPx, 0.);
		var last = this.embeded ? this.tfValues[3] : this.tfValues[4];
		this.tfUnits.set_text(com_watabou_mfcg_scenes_overlays_ScaleBar.units.unit);
		this.tfUnits.set_x(last.get_x() + last.get_width());
		this.tfUnits.set_y(last.get_y());
	}
	, getMinSize: function () {
		return 180;
	}
	, __class__: com_watabou_mfcg_scenes_overlays_ScaleBarNew
});
var com_watabou_mfcg_scenes_overlays_ScaleBarOld = function (embeded) {
	if (embeded == null) {
		embeded = false;
	}
	this.embeded = embeded;
	com_watabou_mfcg_scenes_overlays_ScaleBar.call(this);
	this.tfZero = this.createLabel("0");
	this.addChild(this.tfZero);
	this.tfHalf = this.createLabel();
	this.addChild(this.tfHalf);
	this.tfFull = this.createLabel();
	this.addChild(this.tfFull);
	this.tfUnits = this.createLabel();
	this.addChild(this.tfUnits);
};
$hxClasses["com.watabou.mfcg.scenes.overlays.ScaleBarOld"] = com_watabou_mfcg_scenes_overlays_ScaleBarOld;
com_watabou_mfcg_scenes_overlays_ScaleBarOld.__name__ = "com.watabou.mfcg.scenes.overlays.ScaleBarOld";
com_watabou_mfcg_scenes_overlays_ScaleBarOld.__super__ = com_watabou_mfcg_scenes_overlays_ScaleBar;
com_watabou_mfcg_scenes_overlays_ScaleBarOld.prototype = $extend(com_watabou_mfcg_scenes_overlays_ScaleBar.prototype, {
	update: function (map) {
		com_watabou_mfcg_scenes_overlays_ScaleBar.prototype.update.call(this, map);
		this.get_graphics().beginFill(16711680, 0.0);
		this.get_graphics().drawRect(0, -12, (this.tickPx | 0) + 1, 12);
		this.get_graphics().endFill();
		var tmp = this.get_graphics();
		var unscaled = false;
		if (unscaled == null) {
			unscaled = true;
		}
		tmp.lineStyle(com_watabou_mfcg_mapping_Style.getStrokeWidth(com_watabou_mfcg_mapping_Style.strokeNormal, unscaled), com_watabou_mfcg_mapping_Style.colorDark, 1, true, null, 0, 1);
		this.get_graphics().moveTo(0, -12);
		this.get_graphics().lineTo(0, 0);
		this.get_graphics().lineTo(this.tickPx, 0);
		this.get_graphics().lineTo(this.tickPx, -12);
		this.get_graphics().moveTo(this.tickPx * 0.25, 0);
		this.get_graphics().lineTo(this.tickPx * 0.25, -6.);
		this.get_graphics().moveTo(this.tickPx * 0.50, 0);
		this.get_graphics().lineTo(this.tickPx * 0.50, -12.);
		this.get_graphics().moveTo(this.tickPx * 0.75, 0);
		this.get_graphics().lineTo(this.tickPx * 0.75, -6.);
		this.tfZero.set_x(this.embeded ? -2 : -this.tfZero.get_width() / 2);
		this.tfZero.set_y(-12 - this.tfZero.get_height());
		this.tfHalf.set_text(Std.string(this.tickUnit / 2));
		this.tfHalf.set_x(this.tickPx / 2 - this.tfHalf.get_width() / 2);
		this.tfHalf.set_y(-12 - this.tfHalf.get_height());
		this.tfFull.set_text(Std.string(this.tickUnit));
		this.tfFull.set_x(this.tickPx - (this.embeded ? this.tfFull.get_width() - 2 : this.tfFull.get_width() / 2));
		this.tfFull.set_y(-12 - this.tfFull.get_height());
		this.tfUnits.set_text(com_watabou_mfcg_scenes_overlays_ScaleBar.units.unit);
		this.tfUnits.set_x(this.embeded ? this.tfZero.get_x() + this.tfZero.get_width() : this.tfFull.get_x() + this.tfFull.get_width());
		this.tfUnits.set_y(this.tfFull.get_y());
	}
	, getMinSize: function () {
		return 150;
	}
	, __class__: com_watabou_mfcg_scenes_overlays_ScaleBarOld
});
var com_watabou_mfcg_scenes_overlays_ScaleBarOverlay = function (scene) {
	this.scalebar = com_watabou_mfcg_scenes_overlays_ScaleBar.create();
	com_watabou_mfcg_scenes_overlays_Overlay.call(this, scene);
	this.addChild(this.scalebar);
};
$hxClasses["com.watabou.mfcg.scenes.overlays.ScaleBarOverlay"] = com_watabou_mfcg_scenes_overlays_ScaleBarOverlay;
com_watabou_mfcg_scenes_overlays_ScaleBarOverlay.__name__ = "com.watabou.mfcg.scenes.overlays.ScaleBarOverlay";
com_watabou_mfcg_scenes_overlays_ScaleBarOverlay.__super__ = com_watabou_mfcg_scenes_overlays_Overlay;
com_watabou_mfcg_scenes_overlays_ScaleBarOverlay.prototype = $extend(com_watabou_mfcg_scenes_overlays_Overlay.prototype, {
	onContext: function (context) {
		var _gthis = this;
		var addSystem = function (name, system) {
			context.addItem(name, function () {
				com_watabou_mfcg_scenes_overlays_ScaleBar.system = system;
				_gthis.scalebar.update();
			}, com_watabou_mfcg_scenes_overlays_ScaleBar.system == system);
		};
		addSystem("Metric units", com_watabou_mfcg_scenes_overlays_ScaleBar.METRIC);
		addSystem("Imperial units", com_watabou_mfcg_scenes_overlays_ScaleBar.IMPERIAL);
		var addView = function (name, view) {
			context.addItem(name, function () {
				com_watabou_mfcg_scenes_overlays_ScaleBar.sbClass = view;
				_gthis.replace();
			}, com_watabou_mfcg_scenes_overlays_ScaleBar.sbClass == view);
		};
		addView("Default style", com_watabou_mfcg_scenes_overlays_ScaleBarOld);
		addView("Alternative style", com_watabou_mfcg_scenes_overlays_ScaleBarNew);
		context.addItem("Hide", function () {
			com_watabou_system_State.set("scale_bar", false);
		});
	}
	, replace: function () {
		if (this.scalebar != null) {
			this.removeChild(this.scalebar);
		}
		this.scalebar = com_watabou_mfcg_scenes_overlays_ScaleBar.create();
		this.addChild(this.scalebar);
		this.layout();
	}
	, update: function (model) {
		this.scalebar.update(this.scene.map);
	}
	, layout: function () {
		this.update(null);
		this.scalebar.set_x(com_watabou_mfcg_scenes_overlays_Overlay.MARGIN);
		this.scalebar.set_y(this.rHeight - com_watabou_mfcg_scenes_overlays_Overlay.MARGIN);
	}
	, __class__: com_watabou_mfcg_scenes_overlays_ScaleBarOverlay
});
var com_watabou_mfcg_scenes_overlays_Title = function () {
	openfl_display_Sprite.call(this);
	this.text = new openfl_text_TextField();
	this.text.set_defaultTextFormat(com_watabou_mfcg_ui_Text.getFormat("font_title", com_watabou_mfcg_ui_Text.fontTitle, com_watabou_mfcg_mapping_Style.colorDark));
	this.text.set_autoSize(1);
	this.addChild(this.text);
	this.mouseEnabled = true;
	this.mouseChildren = false;
	this.filterOn(true);
	this.addEventListener("mouseDown", $bind(this, this.onClick));
};
$hxClasses["com.watabou.mfcg.scenes.overlays.Title"] = com_watabou_mfcg_scenes_overlays_Title;
com_watabou_mfcg_scenes_overlays_Title.__name__ = "com.watabou.mfcg.scenes.overlays.Title";
com_watabou_mfcg_scenes_overlays_Title.__super__ = openfl_display_Sprite;
com_watabou_mfcg_scenes_overlays_Title.prototype = $extend(openfl_display_Sprite.prototype, {
	setText: function (txt) {
		this.text.set_text(txt);
		this.text.set_x(-this.text.get_width() / 2);
		this.text.set_y(-this.text.get_height() / 2);
	}
	, edit: function (model) {
		var _gthis = this;
		this.set_visible(false);
		new com_watabou_mfcg_ui_EditInPlace(model.name, this.text.get_defaultTextFormat(), new openfl_geom_Point(this.get_x(), this.get_y()), this.parent, null, function (value) {
			_gthis.set_visible(true);
			model.setName(value, true);
		}, function () {
			_gthis.set_visible(true);
		});
	}
	, onClick: function (e) {
		var model = com_watabou_mfcg_model_City.instance;
		if (e.shiftKey) {
			model.setName(model.rerollName());
		} else {
			this.edit(model);
		}
	}
	, filterOn: function (state) {
		if (this.outline != null) {
			this.removeChild(this.outline);
			this.outline = null;
		}
		var unscaled = false;
		if (unscaled == null) {
			unscaled = true;
		}
		var th = com_watabou_mfcg_mapping_Style.getStrokeWidth(com_watabou_mfcg_mapping_Style.strokeThick, unscaled);
		if (state) {
			this.set_filters([new openfl_filters_GlowFilter(com_watabou_mfcg_mapping_Style.colorPaper, 1, th * 2, th * 2, 100)]);
		} else {
			this.set_filters([]);
			this.outline = new com_watabou_mfcg_ui_Outline(this.text, com_watabou_mfcg_mapping_Style.colorPaper, th, 8);
			this.addChildAt(this.outline, 0);
		}
	}
	, __class__: com_watabou_mfcg_scenes_overlays_Title
});
var com_watabou_mfcg_scenes_overlays_TitleOverlay = function (scene) {
	this.title = new com_watabou_mfcg_scenes_overlays_Title();
	com_watabou_mfcg_scenes_overlays_Overlay.call(this, scene);
	this.addChild(this.title);
	com_watabou_mfcg_model_ModelDispatcher.titleChanged.add($bind(this, this.onChanged));
};
$hxClasses["com.watabou.mfcg.scenes.overlays.TitleOverlay"] = com_watabou_mfcg_scenes_overlays_TitleOverlay;
com_watabou_mfcg_scenes_overlays_TitleOverlay.__name__ = "com.watabou.mfcg.scenes.overlays.TitleOverlay";
com_watabou_mfcg_scenes_overlays_TitleOverlay.__super__ = com_watabou_mfcg_scenes_overlays_Overlay;
com_watabou_mfcg_scenes_overlays_TitleOverlay.prototype = $extend(com_watabou_mfcg_scenes_overlays_Overlay.prototype, {
	layout: function () {
		this.title.set_x(this.rWidth / 2);
		this.title.set_y(com_watabou_mfcg_scenes_overlays_Overlay.MARGIN + this.title.get_height() / 2);
	}
	, update: function (model) {
		com_watabou_mfcg_scenes_overlays_Overlay.prototype.update.call(this, model);
		this.title.setText(model.name);
	}
	, onChanged: function (value) {
		this.title.setText(value);
	}
	, onDestroy: function () {
		com_watabou_mfcg_scenes_overlays_Overlay.prototype.onDestroy.call(this);
		com_watabou_mfcg_model_ModelDispatcher.titleChanged.remove($bind(this, this.onChanged));
	}
	, onContext: function (context) {
		var _gthis = this;
		context.addItem("Edit", function () {
			_gthis.title.edit(_gthis.model);
		});
		context.addItem("Reroll", function () {
			_gthis.model.setName(_gthis.model.rerollName());
			_gthis.title.setText(_gthis.model.name);
		});
		context.addItem("Hide", function () {
			com_watabou_system_State.set("city_name", false);
		});
	}
	, exportPNG: function (state) {
		this.title.filterOn(!state);
	}
	, __class__: com_watabou_mfcg_scenes_overlays_TitleOverlay
});
var com_watabou_mfcg_scenes_tools_WarpTool = function (scene) {
	this.affectedPatches = [];
	this.affectedNodes = new haxe_ds_ObjectMap();
	this.softness = 0.5;
	this.radius = 20.0;
	this.scene = scene;
};
$hxClasses["com.watabou.mfcg.scenes.tools.WarpTool"] = com_watabou_mfcg_scenes_tools_WarpTool;
com_watabou_mfcg_scenes_tools_WarpTool.__name__ = "com.watabou.mfcg.scenes.tools.WarpTool";
com_watabou_mfcg_scenes_tools_WarpTool.prototype = {
	getName: function () {
		return "Unknown tool";
	}
	, activate: function () {
	}
	, onPress: function (x, y) {
		this.startx = this.prevx = this.curx = x;
		this.starty = this.prevy = this.cury = y;
	}
	, onRelease: function () {
		if (this.startx != this.curx || this.starty != this.cury) {
			this.scene.submit(this.scene.model.patches, true);
		}
	}
	, onMove: function (x, y) {
	}
	, onDrag: function (x, y) {
		this.prevx = this.curx;
		this.prevy = this.cury;
		this.curx = x;
		this.cury = y;
	}
	, onWheel: function (x, y, delta) {
		this.radius = com_watabou_utils_MathUtils.gate(this.radius * Math.pow(1.05, delta == 0 ? 0 : delta < 0 ? -1 : 1), 2, 100);
		this.scene.updateBrush(this.radius, 1 - this.softness);
		this.onMove(x, y);
	}
	, onKey: function (key) {
		switch (key) {
			case 107: case 187:
				this.radius = com_watabou_utils_MathUtils.gate(this.radius * 1.05, 2, 100);
				this.scene.updateBrush(this.radius, 1 - this.softness);
				return true;
			case 109: case 189:
				this.radius = com_watabou_utils_MathUtils.gate(this.radius / 1.05, 2, 100);
				this.scene.updateBrush(this.radius, 1 - this.softness);
				return true;
			default:
				return false;
		}
	}
	, affect: function (p) {
		this.affectedNodes = new haxe_ds_ObjectMap();
		this.affectedPatches = [];
		var node = this.scene.nodePatches.keys();
		while (node.hasNext()) {
			var node1 = node.next();
			var d = openfl_geom_Point.distance(p, node1);
			if (d < this.radius) {
				var v = Math.min(1, (1 - d / this.radius) / this.softness);
				this.affectedNodes.set(node1, v);
				com_watabou_utils_ArrayExtender.addAll(this.affectedPatches, this.scene.nodePatches.h[node1.__id__]);
			}
		}
	}
	, updateMesh: function () {
		var _gthis = this;
		this.scene.clearMesh();
		var _g = 0;
		var _g1 = this.affectedPatches;
		while (_g < _g1.length) {
			var patch = _g1[_g];
			++_g;
			com_watabou_geom_polygons_PolyAccess.forEdge(patch.shape, function (v1, v2) {
				var a1 = _gthis.affectedNodes.h.__keys__[v1.__id__] != null ? _gthis.affectedNodes.h[v1.__id__] : 0.0;
				var a2 = _gthis.affectedNodes.h.__keys__[v2.__id__] != null ? _gthis.affectedNodes.h[v2.__id__] : 0.0;
				_gthis.scene.drawEdge(v1, v2, 2.0 * (a1 + a2) / 2);
			});
		}
		var n = this.affectedNodes.keys();
		while (n.hasNext()) {
			var n1 = n.next();
			this.scene.drawNode(n1, this.affectedNodes.h[n1.__id__] * 4.0);
		}
	}
	, showBrush: function () {
		this.scene.updateBrush(this.radius, 1 - this.softness);
	}
	, hideBrush: function () {
		this.scene.updateBrush(0);
	}
	, __class__: com_watabou_mfcg_scenes_tools_WarpTool
};
var com_watabou_mfcg_scenes_tools_BloatTool = function (scene) {
	this.modifiedPatches = [];
	com_watabou_mfcg_scenes_tools_WarpTool.call(this, scene);
};
$hxClasses["com.watabou.mfcg.scenes.tools.BloatTool"] = com_watabou_mfcg_scenes_tools_BloatTool;
com_watabou_mfcg_scenes_tools_BloatTool.__name__ = "com.watabou.mfcg.scenes.tools.BloatTool";
com_watabou_mfcg_scenes_tools_BloatTool.__super__ = com_watabou_mfcg_scenes_tools_WarpTool;
com_watabou_mfcg_scenes_tools_BloatTool.prototype = $extend(com_watabou_mfcg_scenes_tools_WarpTool.prototype, {
	getName: function () {
		return "Bloat";
	}
	, activate: function () {
		this.radius = com_watabou_mfcg_scenes_tools_BloatTool.brushRadius;
		this.softness = 0.0;
		this.scene.updateBrush(this.radius, 1 - this.softness);
	}
	, onMove: function (x, y) {
		this.affect(new openfl_geom_Point(x, y));
		this.updateMesh();
	}
	, onDrag: function (x, y) {
		com_watabou_mfcg_scenes_tools_WarpTool.prototype.onDrag.call(this, x, y);
		var dx = this.curx - this.prevx;
		var dy = this.cury - this.prevy;
		var d = Math.sqrt(dx * dx + dy * dy);
		var k = Math.min(1, 0.5 * d / this.radius);
		var c = new openfl_geom_Point(x, y);
		this.bloat(c, k);
		com_watabou_utils_ArrayExtender.addAll(this.modifiedPatches, this.affectedPatches);
		this.affect(c);
		this.updateMesh();
	}
	, bloat: function (c, k) {
		var map = this.affectedNodes;
		var _g_map = map;
		var _g_keys = map.keys();
		while (_g_keys.hasNext()) {
			var key = _g_keys.next();
			var _g1_value = _g_map.get(key);
			var _g1_key = key;
			var p = _g1_key;
			var _ = _g1_value;
			var p1 = p.subtract(c);
			var f = Math.pow(openfl_geom_Point.distance(p, c) / this.radius, -k);
			p1.x *= f;
			p1.y *= f;
			p1.x += c.x;
			p1.y += c.y;
			com_watabou_utils_PointExtender.set(p, p1);
		}
	}
	, onPress: function (x, y) {
		com_watabou_mfcg_scenes_tools_WarpTool.prototype.onPress.call(this, x, y);
		this.modifiedPatches = [];
	}
	, onRelease: function () {
		com_watabou_mfcg_scenes_tools_BloatTool.brushRadius = this.radius;
		this.scene.submit(this.modifiedPatches, true);
	}
	, __class__: com_watabou_mfcg_scenes_tools_BloatTool
});
var com_watabou_mfcg_scenes_tools_DisplaceTool = function (scene) {
	com_watabou_mfcg_scenes_tools_WarpTool.call(this, scene);
};
$hxClasses["com.watabou.mfcg.scenes.tools.DisplaceTool"] = com_watabou_mfcg_scenes_tools_DisplaceTool;
com_watabou_mfcg_scenes_tools_DisplaceTool.__name__ = "com.watabou.mfcg.scenes.tools.DisplaceTool";
com_watabou_mfcg_scenes_tools_DisplaceTool.__super__ = com_watabou_mfcg_scenes_tools_WarpTool;
com_watabou_mfcg_scenes_tools_DisplaceTool.prototype = $extend(com_watabou_mfcg_scenes_tools_WarpTool.prototype, {
	getName: function () {
		return "Displace";
	}
	, activate: function () {
		this.radius = com_watabou_mfcg_scenes_tools_DisplaceTool.brushRadius;
		this.scene.updateBrush(this.radius, 1 - this.softness);
	}
	, onMove: function (x, y) {
		this.affect(new openfl_geom_Point(x, y));
		this.updateMesh();
	}
	, onDrag: function (x, y) {
		com_watabou_mfcg_scenes_tools_WarpTool.prototype.onDrag.call(this, x, y);
		var dx = this.curx - this.prevx;
		var dy = this.cury - this.prevy;
		var map = this.affectedNodes;
		var _g_map = map;
		var _g_keys = map.keys();
		while (_g_keys.hasNext()) {
			var key = _g_keys.next();
			var _g1_value = _g_map.get(key);
			var _g1_key = key;
			var p = _g1_key;
			var value = _g1_value;
			p.x += dx * value;
			p.y += dy * value;
		}
		this.updateMesh();
	}
	, onPress: function (x, y) {
		com_watabou_mfcg_scenes_tools_WarpTool.prototype.onPress.call(this, x, y);
		this.scene.updateBrush(0);
	}
	, onRelease: function () {
		com_watabou_mfcg_scenes_tools_DisplaceTool.brushRadius = this.radius;
		this.scene.updateBrush(this.radius, 1 - this.softness);
		if (this.startx == this.curx && this.starty == this.cury) {
			return;
		}
		var updateTideLine = false;
		var node = this.affectedNodes.keys();
		while (node.hasNext()) {
			var node1 = node.next();
			if (this.scene.model.shore.indexOf(node1) != -1) {
				updateTideLine = true;
				break;
			}
		}
		this.scene.submit(this.affectedPatches, updateTideLine);
	}
	, __class__: com_watabou_mfcg_scenes_tools_DisplaceTool
});
var com_watabou_mfcg_scenes_tools_EqualizeTool = function (scene) {
	com_watabou_mfcg_scenes_tools_WarpTool.call(this, scene);
};
$hxClasses["com.watabou.mfcg.scenes.tools.EqualizeTool"] = com_watabou_mfcg_scenes_tools_EqualizeTool;
com_watabou_mfcg_scenes_tools_EqualizeTool.__name__ = "com.watabou.mfcg.scenes.tools.EqualizeTool";
com_watabou_mfcg_scenes_tools_EqualizeTool.pointInPoly = function (pt, poly) {
	var count = 0;
	var x2 = pt.x;
	var y2 = pt.y;
	var dx2 = 1;
	var dy2 = 0;
	var p1 = poly[poly.length - 1];
	var _g = 0;
	var _g1 = poly.length;
	while (_g < _g1) {
		var i = _g++;
		var p0 = p1;
		p1 = poly[i];
		var x1 = p0.x;
		var y1 = p0.y;
		var dx1 = p1.x - x1;
		var dy1 = p1.y - y1;
		var t = com_watabou_geom_GeomUtils.intersectLines(x1, y1, dx1, dy1, x2, y2, dx2, dy2);
		if (t != null && t.x >= 0 && t.x <= 1 && t.y >= 0) {
			++count;
		}
	}
	return (count & 1) != 0;
};
com_watabou_mfcg_scenes_tools_EqualizeTool.__super__ = com_watabou_mfcg_scenes_tools_WarpTool;
com_watabou_mfcg_scenes_tools_EqualizeTool.prototype = $extend(com_watabou_mfcg_scenes_tools_WarpTool.prototype, {
	getName: function () {
		return "Equalize";
	}
	, activate: function () {
		this.patch = null;
		this.scene.updateBrush(0);
	}
	, onMove: function (x, y) {
		var p = this.findPatch(x, y);
		if (this.patch != p) {
			this.patch = p;
			this.affectedNodes = new haxe_ds_ObjectMap();
			this.affectedPatches = [];
			if (this.patch != null) {
				this.shape = this.patch.shape;
				this.len = this.shape.length;
				var _g = 0;
				var _g1 = this.shape;
				while (_g < _g1.length) {
					var p = _g1[_g];
					++_g;
					this.affectedNodes.set(p, 1.0);
					com_watabou_utils_ArrayExtender.addAll(this.affectedPatches, this.scene.nodePatches.h[p.__id__]);
				}
			}
			this.updateMesh();
		}
	}
	, onDrag: function (x, y) {
		com_watabou_mfcg_scenes_tools_WarpTool.prototype.onDrag.call(this, x, y);
		if (this.patch != null) {
			var dx = this.curx - this.prevx;
			var dy = this.cury - this.prevy;
			var d = Math.sqrt(dx * dx + dy * dy);
			var k = Math.min(1, 2 * d / com_watabou_geom_polygons_PolyCore.perimeter(this.shape));
			this.equalize(k);
			this.updateMesh();
		}
	}
	, equalize: function (k) {
		var _g = [];
		var _g1 = 0;
		var _g2 = this.len;
		while (_g1 < _g2) {
			var i = _g1++;
			var v0 = this.shape[(i + this.len - 1) % this.len];
			var v1 = this.shape[i];
			var v2 = this.shape[(i + 1) % this.len];
			var v = v2.subtract(v0);
			var d = (v.x * v1.y - v.y * v1.x - v2.x * v0.y + v2.y * v0.x) / v.get_length();
			var tmp = com_watabou_geom_GeomUtils.lerp(v0, v2);
			var p = new openfl_geom_Point(-v.y, v.x);
			var length = d;
			if (length == null) {
				length = 1;
			}
			p = p.clone();
			p.normalize(length);
			_g.push(tmp.add(p));
		}
		var sh = _g;
		var _g = 0;
		var _g1 = this.len;
		while (_g < _g1) {
			var i = _g++;
			com_watabou_utils_PointExtender.set(this.shape[i], com_watabou_geom_GeomUtils.lerp(this.shape[i], sh[i], k));
		}
	}
	, onRelease: function () {
		var updateTideLine = false;
		var node = this.affectedNodes.keys();
		while (node.hasNext()) {
			var node1 = node.next();
			if (this.scene.model.shore.indexOf(node1) != -1) {
				updateTideLine = true;
				break;
			}
		}
		this.scene.submit(this.affectedPatches, updateTideLine);
		this.onMove(this.curx, this.cury);
	}
	, updateMesh: function () {
		this.scene.clearMesh();
		if (this.patch == null) {
			return;
		}
		var _g = 0;
		var _g1 = this.len;
		while (_g < _g1) {
			var i = _g++;
			var v0 = this.shape[i];
			var v1 = this.shape[(i + 1) % this.len];
			this.scene.drawEdge(v0, v1, 2.0);
			this.scene.drawNode(v0, 4.0);
		}
	}
	, findPatch: function (x, y) {
		var p = new openfl_geom_Point(x, y);
		var _g = 0;
		var _g1 = this.scene.model.dcel.faces;
		while (_g < _g1.length) {
			var face = _g1[_g];
			++_g;
			if (com_watabou_mfcg_scenes_tools_EqualizeTool.pointInPoly(p, face.data.shape)) {
				return face.data;
			}
		}
		return null;
	}
	, __class__: com_watabou_mfcg_scenes_tools_EqualizeTool
});
var com_watabou_mfcg_scenes_tools_LiquifyTool = function (scene) {
	this.modifiedPatches = [];
	com_watabou_mfcg_scenes_tools_WarpTool.call(this, scene);
};
$hxClasses["com.watabou.mfcg.scenes.tools.LiquifyTool"] = com_watabou_mfcg_scenes_tools_LiquifyTool;
com_watabou_mfcg_scenes_tools_LiquifyTool.__name__ = "com.watabou.mfcg.scenes.tools.LiquifyTool";
com_watabou_mfcg_scenes_tools_LiquifyTool.__super__ = com_watabou_mfcg_scenes_tools_WarpTool;
com_watabou_mfcg_scenes_tools_LiquifyTool.prototype = $extend(com_watabou_mfcg_scenes_tools_WarpTool.prototype, {
	getName: function () {
		return "Liquify";
	}
	, activate: function () {
		this.radius = com_watabou_mfcg_scenes_tools_LiquifyTool.brushRadius;
		this.softness = 0.0;
		this.scene.updateBrush(this.radius, 1 - this.softness);
	}
	, onMove: function (x, y) {
		this.affect(new openfl_geom_Point(x, y));
		this.updateMesh();
	}
	, onDrag: function (x, y) {
		com_watabou_mfcg_scenes_tools_WarpTool.prototype.onDrag.call(this, x, y);
		var dx = this.curx - this.prevx;
		var dy = this.cury - this.prevy;
		var map = this.affectedNodes;
		var _g_map = map;
		var _g_keys = map.keys();
		while (_g_keys.hasNext()) {
			var key = _g_keys.next();
			var _g1_value = _g_map.get(key);
			var _g1_key = key;
			var p = _g1_key;
			var value = _g1_value;
			p.x += dx * value * 0.5;
			p.y += dy * value * 0.5;
		}
		com_watabou_utils_ArrayExtender.addAll(this.modifiedPatches, this.affectedPatches);
		this.affect(new openfl_geom_Point(x, y));
		this.updateMesh();
	}
	, onPress: function (x, y) {
		com_watabou_mfcg_scenes_tools_WarpTool.prototype.onPress.call(this, x, y);
		this.modifiedPatches = [];
	}
	, onRelease: function () {
		com_watabou_mfcg_scenes_tools_LiquifyTool.brushRadius = this.radius;
		this.scene.submit(this.modifiedPatches, true);
	}
	, affect: function (p) {
		this.affectedNodes = new haxe_ds_ObjectMap();
		this.affectedPatches = [];
		var node = this.scene.nodePatches.keys();
		while (node.hasNext()) {
			var node1 = node.next();
			var d = openfl_geom_Point.distance(p, node1);
			if (d < this.radius) {
				var v = 1 - d / this.radius;
				this.affectedNodes.set(node1, v);
				com_watabou_utils_ArrayExtender.addAll(this.affectedPatches, this.scene.nodePatches.h[node1.__id__]);
			}
		}
	}
	, __class__: com_watabou_mfcg_scenes_tools_LiquifyTool
});
var com_watabou_mfcg_scenes_tools_MeasureTool = function (scene) {
	com_watabou_mfcg_scenes_tools_WarpTool.call(this, scene);
};
$hxClasses["com.watabou.mfcg.scenes.tools.MeasureTool"] = com_watabou_mfcg_scenes_tools_MeasureTool;
com_watabou_mfcg_scenes_tools_MeasureTool.__name__ = "com.watabou.mfcg.scenes.tools.MeasureTool";
com_watabou_mfcg_scenes_tools_MeasureTool.__super__ = com_watabou_mfcg_scenes_tools_WarpTool;
com_watabou_mfcg_scenes_tools_MeasureTool.prototype = $extend(com_watabou_mfcg_scenes_tools_WarpTool.prototype, {
	getName: function () {
		return "Measure";
	}
	, activate: function () {
		this.scene.updateBrush(0);
		this.scene.clearMesh();
	}
	, onPress: function (x, y) {
		com_watabou_mfcg_scenes_tools_WarpTool.prototype.onPress.call(this, x, y);
		this.start = new openfl_geom_Point(x, y);
		this.cur = new openfl_geom_Point(x, y);
		this.updateMesh();
	}
	, onRelease: function () {
		if (this.startx != this.curx || this.starty != this.cury) {
			com_watabou_coogee_ui_Toast.show("" + Math.round(4 * openfl_geom_Point.distance(this.start, this.cur)) + " m");
		} else {
			this.scene.clearMesh();
		}
	}
	, onDrag: function (x, y) {
		com_watabou_mfcg_scenes_tools_WarpTool.prototype.onDrag.call(this, x, y);
		this.cur.setTo(x, y);
		this.updateMesh();
	}
	, updateMesh: function () {
		this.scene.clearMesh();
		this.scene.drawEdge(this.start, this.cur, 2.0);
		this.scene.drawNode(this.start, 4.0);
		this.scene.drawNode(this.cur, 4.0);
	}
	, __class__: com_watabou_mfcg_scenes_tools_MeasureTool
});
var com_watabou_mfcg_scenes_tools_PinchTool = function (scene) {
	com_watabou_mfcg_scenes_tools_BloatTool.call(this, scene);
};
$hxClasses["com.watabou.mfcg.scenes.tools.PinchTool"] = com_watabou_mfcg_scenes_tools_PinchTool;
com_watabou_mfcg_scenes_tools_PinchTool.__name__ = "com.watabou.mfcg.scenes.tools.PinchTool";
com_watabou_mfcg_scenes_tools_PinchTool.__super__ = com_watabou_mfcg_scenes_tools_BloatTool;
com_watabou_mfcg_scenes_tools_PinchTool.prototype = $extend(com_watabou_mfcg_scenes_tools_BloatTool.prototype, {
	getName: function () {
		return "Pinch";
	}
	, bloat: function (c, k) {
		com_watabou_mfcg_scenes_tools_BloatTool.prototype.bloat.call(this, c, -k);
	}
	, __class__: com_watabou_mfcg_scenes_tools_PinchTool
});
var com_watabou_mfcg_scenes_tools_RelaxTool = function (scene) {
	this.modifiedPatches = [];
	com_watabou_mfcg_scenes_tools_WarpTool.call(this, scene);
};
$hxClasses["com.watabou.mfcg.scenes.tools.RelaxTool"] = com_watabou_mfcg_scenes_tools_RelaxTool;
com_watabou_mfcg_scenes_tools_RelaxTool.__name__ = "com.watabou.mfcg.scenes.tools.RelaxTool";
com_watabou_mfcg_scenes_tools_RelaxTool.__super__ = com_watabou_mfcg_scenes_tools_WarpTool;
com_watabou_mfcg_scenes_tools_RelaxTool.prototype = $extend(com_watabou_mfcg_scenes_tools_WarpTool.prototype, {
	getName: function () {
		return "Relax";
	}
	, activate: function () {
		this.radius = com_watabou_mfcg_scenes_tools_RelaxTool.brushRadius;
		this.scene.updateBrush(this.radius, 1 - this.softness);
	}
	, onMove: function (x, y) {
		this.affect(new openfl_geom_Point(x, y));
		this.updateMesh();
	}
	, onDrag: function (x, y) {
		com_watabou_mfcg_scenes_tools_WarpTool.prototype.onDrag.call(this, x, y);
		var dx = this.curx - this.prevx;
		var dy = this.cury - this.prevy;
		var d = Math.sqrt(dx * dx + dy * dy);
		var k = Math.min(1, d / this.radius);
		this.relax(k);
		com_watabou_utils_ArrayExtender.addAll(this.modifiedPatches, this.affectedPatches);
		this.affect(new openfl_geom_Point(x, y));
		this.updateMesh();
	}
	, relax: function (k) {
		var relaxed = new haxe_ds_ObjectMap();
		var map = this.affectedNodes;
		var _g_map = map;
		var _g_keys = map.keys();
		while (_g_keys.hasNext()) {
			var key = _g_keys.next();
			var _g1_value = _g_map.get(key);
			var _g1_key = key;
			var p = _g1_key;
			var value = _g1_value;
			var v = this.scene.model.dcel.vertices.h[p.__id__];
			var sum = new openfl_geom_Point();
			var inner = true;
			var _g = 0;
			var _g1 = v.edges;
			while (_g < _g1.length) {
				var e = _g1[_g];
				++_g;
				if (e.twin == null) {
					inner = false;
					break;
				} else {
					var q = e.next.origin.point;
					sum.x += q.x;
					sum.y += q.y;
				}
			}
			if (inner) {
				var p1 = v.point;
				var f = 1 / v.edges.length;
				sum.x *= f;
				sum.y *= f;
				sum.x -= p1.x;
				sum.y -= p1.y;
				relaxed.set(p1, sum);
			}
		}
		var map = relaxed;
		var _g_map = map;
		var _g_keys = map.keys();
		while (_g_keys.hasNext()) {
			var key = _g_keys.next();
			var _g1_value = _g_map.get(key);
			var _g1_key = key;
			var p = _g1_key;
			var d = _g1_value;
			var k1 = k * this.affectedNodes.h[p.__id__];
			p.x += d.x * k1;
			p.y += d.y * k1;
		}
	}
	, onPress: function (x, y) {
		com_watabou_mfcg_scenes_tools_WarpTool.prototype.onPress.call(this, x, y);
		this.modifiedPatches = [];
	}
	, onRelease: function () {
		com_watabou_mfcg_scenes_tools_RelaxTool.brushRadius = this.radius;
		this.scene.submit(this.modifiedPatches, true);
	}
	, __class__: com_watabou_mfcg_scenes_tools_RelaxTool
});
var com_watabou_mfcg_scenes_tools_RotateTool = function (scene) {
	com_watabou_mfcg_scenes_tools_WarpTool.call(this, scene);
	var _g = [];
	var _g1 = 0;
	var _g2 = scene.model.dcel.edges;
	while (_g1 < _g2.length) {
		var v = _g2[_g1];
		++_g1;
		if (v.data != null) {
			_g.push(v);
		}
	}
	this.segments = _g;
};
$hxClasses["com.watabou.mfcg.scenes.tools.RotateTool"] = com_watabou_mfcg_scenes_tools_RotateTool;
com_watabou_mfcg_scenes_tools_RotateTool.__name__ = "com.watabou.mfcg.scenes.tools.RotateTool";
com_watabou_mfcg_scenes_tools_RotateTool.__super__ = com_watabou_mfcg_scenes_tools_WarpTool;
com_watabou_mfcg_scenes_tools_RotateTool.prototype = $extend(com_watabou_mfcg_scenes_tools_WarpTool.prototype, {
	getName: function () {
		return "Rotate";
	}
	, activate: function () {
		this.scene.updateBrush(0);
		this.updateMesh();
	}
	, onDrag: function (x, y) {
		com_watabou_mfcg_scenes_tools_WarpTool.prototype.onDrag.call(this, x, y);
		var d1 = new openfl_geom_Point(this.prevx, this.prevy).get_length();
		var d2 = new openfl_geom_Point(this.curx, this.cury).get_length();
		var cosA = (this.prevx * this.curx + this.prevy * this.cury) / (d1 * d2);
		var sinA = (this.cury * this.prevx - this.curx * this.prevy) / (d1 * d2);
		var v = this.scene.prevState.keys();
		while (v.hasNext()) {
			var v1 = v.next();
			var vx = v1.x * cosA - v1.y * sinA;
			var vy = v1.y * cosA + v1.x * sinA;
			v1.setTo(vx, vy);
		}
		this.updateMesh();
	}
	, updateMesh: function () {
		this.scene.clearMesh();
		var _g = 0;
		var _g1 = this.segments;
		while (_g < _g1.length) {
			var s = _g1[_g];
			++_g;
			this.scene.drawEdge(s.origin.point, s.next.origin.point, 2.0);
		}
	}
	, __class__: com_watabou_mfcg_scenes_tools_RotateTool
});
var com_watabou_mfcg_ui_CurvedText = function (text, format) {
	openfl_display_Sprite.call(this);
	this.length = text.length;
	this.symbols = [];
	this.letters = [];
	var _g = 0;
	var _g1 = this.length;
	while (_g < _g1) {
		var i = _g++;
		var symbol = new openfl_display_Sprite();
		this.symbols.push(symbol);
		this.addChild(symbol);
		var tf = new openfl_text_TextField();
		this.letters.push(tf);
		tf.set_selectable(false);
		tf.set_autoSize(1);
		tf.set_defaultTextFormat(format);
		tf.set_text(text.charAt(i));
		tf.set_x(-tf.get_width() / 2);
		tf.set_y(-tf.get_height() / 2);
		symbol.addChild(tf);
	}
	this.basicWidth = this.measure();
	this.arrange(Infinity);
};
$hxClasses["com.watabou.mfcg.ui.CurvedText"] = com_watabou_mfcg_ui_CurvedText;
com_watabou_mfcg_ui_CurvedText.__name__ = "com.watabou.mfcg.ui.CurvedText";
com_watabou_mfcg_ui_CurvedText.__super__ = openfl_display_Sprite;
com_watabou_mfcg_ui_CurvedText.prototype = $extend(openfl_display_Sprite.prototype, {
	measure: function (spacing) {
		if (spacing == null) {
			spacing = 0.0;
		}
		this.pos = [];
		var width = 0.0;
		var prev = 0.0;
		var _g = 0;
		var _g1 = this.length;
		while (_g < _g1) {
			var i = _g++;
			var tf = this.letters[i];
			var curr = tf.getLineMetrics(0).width;
			if (i > 0) {
				width += (curr + prev) / 2 + spacing;
			}
			this.pos.push(width);
			prev = curr;
		}
		var _g = 0;
		var _g1 = this.pos.length;
		while (_g < _g1) {
			var i = _g++;
			this.pos[i] -= width / 2;
		}
		return width;
	}
	, arrange: function (radius) {
		if (radius < Infinity) {
			var _g = 0;
			var _g1 = this.symbols.length;
			while (_g < _g1) {
				var i = _g++;
				var sym = this.symbols[i];
				var angle = this.pos[i] / radius;
				sym.set_x(Math.sin(angle) * radius);
				sym.set_y(Math.cos(angle) * radius - radius);
				sym.set_rotation(-180 / Math.PI * angle);
			}
		} else {
			var _g = 0;
			var _g1 = this.symbols.length;
			while (_g < _g1) {
				var i = _g++;
				this.symbols[i].set_x(this.pos[i]);
			}
		}
	}
	, __class__: com_watabou_mfcg_ui_CurvedText
});
var com_watabou_mfcg_ui_DistrictLabel = function (district) {
	openfl_display_Sprite.call(this);
	this.update(district);
	this.mouseEnabled = true;
	this.mouseChildren = false;
	this.addEventListener("mouseDown", $bind(this, this.onClick));
	this.addEventListener("rightClick", $bind(this, this.onContext));
};
$hxClasses["com.watabou.mfcg.ui.DistrictLabel"] = com_watabou_mfcg_ui_DistrictLabel;
com_watabou_mfcg_ui_DistrictLabel.__name__ = "com.watabou.mfcg.ui.DistrictLabel";
com_watabou_mfcg_ui_DistrictLabel.__super__ = openfl_display_Sprite;
com_watabou_mfcg_ui_DistrictLabel.prototype = $extend(openfl_display_Sprite.prototype, {
	update: function (district) {
		this.removeChildren();
		this.district = district;
		var name = district.name.toUpperCase();
		var width = district.label.span * 0.9;
		var scale = 1 / com_watabou_mfcg_scenes_TownScene.instance.get_mapScale();
		var format = com_watabou_mfcg_ui_Text.getFormat("font_label", com_watabou_mfcg_ui_Text.fontLabel, com_watabou_mfcg_mapping_Style.colorDark, scale);
		this.text = new com_watabou_mfcg_ui_CurvedText(name, format);
		var base = this.text.basicWidth;
		var scale = width / base;
		if (scale < 1) {
			this.set_scaleX(this.set_scaleY(scale));
		} else {
			scale = this.set_scaleX(this.set_scaleY(Math.min(Math.sqrt(scale), 1.5)));
			var space = Math.max(base * scale, width - 10) / scale - base;
			var spacing = space / (name.length - 1);
			this.text.measure(spacing);
		}
		this.text.arrange(district.label.radius);
		this.addChild(this.text);
		this.set_x(district.label.pos.x);
		this.set_y(district.label.pos.y);
		this.set_rotation(district.label.angle * 180 / Math.PI);
		this.filterOn(true);
	}
	, edit: function () {
		var _gthis = this;
		this.set_visible(false);
		var mapScale = com_watabou_mfcg_scenes_TownScene.instance.get_mapScale();
		var format = com_watabou_mfcg_ui_Text.getFormat("font_label", com_watabou_mfcg_ui_Text.fontLabel, com_watabou_mfcg_mapping_Style.colorDark, com_watabou_utils_DisplayObjectExtender.getScale(this) / mapScale);
		var pos = com_watabou_utils_DisplayObjectExtender.convertFrom(this.stage, this.parent, this.district.label.pos);
		new com_watabou_mfcg_ui_EditInPlace(this.district.name, format, pos, this.stage, null, function (value) {
			_gthis.set_visible(true);
			_gthis.district.name = value != "" ? value : "-";
			_gthis.update(_gthis.district);
		}, function () {
			_gthis.set_visible(true);
		});
	}
	, onClick: function (e) {
		if (e.shiftKey) {
			com_watabou_mfcg_model_City.instance.rerollDistricts();
		} else {
			this.edit();
		}
	}
	, onContext: function (e) {
		var context = com_watabou_mfcg_scenes_TownScene.context;
		context.addItem("Edit name", $bind(this, this.edit));
		context.addItem("Reroll all", ($_ = com_watabou_mfcg_model_City.instance, $bind($_, $_.rerollDistricts)));
		context.addItem("Hide all", function () {
			com_watabou_system_State.set("districts", "Hidden");
		});
	}
	, filterOn: function (state) {
		if (this.outline != null) {
			this.removeChild(this.outline);
			this.outline = null;
		}
		if (state) {
			var unscaled = false;
			if (unscaled == null) {
				unscaled = true;
			}
			var thickness = 2 * com_watabou_mfcg_mapping_Style.getStrokeWidth(com_watabou_mfcg_mapping_Style.strokeThick, unscaled);
			this.set_filters([new openfl_filters_GlowFilter(com_watabou_mfcg_mapping_Style.colorPaper, 1, thickness, thickness, 100)]);
		} else {
			this.set_filters([]);
			var thickness = com_watabou_mfcg_mapping_Style.getStrokeWidth(com_watabou_mfcg_mapping_Style.strokeThick, true);
			this.outline = new com_watabou_mfcg_ui_Outline(this.text, com_watabou_mfcg_mapping_Style.colorPaper, thickness, 8);
			this.addChildAt(this.outline, 0);
		}
	}
	, __class__: com_watabou_mfcg_ui_DistrictLabel
});
var com_watabou_mfcg_ui_EditInPlace = function (value, format, pos, parent, align, onSubmit, onCancel) {
	openfl_text_TextField.call(this);
	this.onSubmit = onSubmit;
	this.onCancel = onCancel;
	this.set_defaultTextFormat(format);
	this.set_type(1);
	this.set_text(value);
	this.set_border(true);
	this.set_borderColor(com_watabou_mfcg_mapping_Style.colorDark);
	this.set_background(true);
	this.set_backgroundColor(com_watabou_mfcg_mapping_Style.colorPaper);
	this.set_autoSize(align != null ? align : 0);
	this.addEventListener("focusOut", $bind(this, this.onFocusOut));
	this.addEventListener("keyDown", $bind(this, this.onKeyDown));
	this.addEventListener("keyUp", $bind(this, this.onKeyUp));
	parent.addChild(this);
	this.stage.set_focus(this);
	this.set_x(pos.x - this.get_width() / 2);
	this.set_y(pos.y - this.get_height() / 2);
};
$hxClasses["com.watabou.mfcg.ui.EditInPlace"] = com_watabou_mfcg_ui_EditInPlace;
com_watabou_mfcg_ui_EditInPlace.__name__ = "com.watabou.mfcg.ui.EditInPlace";
com_watabou_mfcg_ui_EditInPlace.fromTextField = function (tf, parent, align, onSubmit) {
	var format = tf.get_defaultTextFormat();
	var srcScale = tf.get_scaleX();
	format.size = format.size * srcScale | 0;
	var p = parent.globalToLocal(tf.localToGlobal(new openfl_geom_Point(tf.get_width() / 2 / srcScale, tf.get_height() / 2 / srcScale)));
	tf.set_visible(false);
	return new com_watabou_mfcg_ui_EditInPlace(tf.get_text(), format, p, parent, align, function (value) {
		tf.set_visible(true);
		onSubmit(value);
	}, function () {
		tf.set_visible(true);
	});
};
com_watabou_mfcg_ui_EditInPlace.__super__ = openfl_text_TextField;
com_watabou_mfcg_ui_EditInPlace.prototype = $extend(openfl_text_TextField.prototype, {
	onFocusOut: function (e) {
		this.submit();
	}
	, onKeyDown: function (e) {
		switch (e.keyCode) {
			case 13:
				this.submit();
				break;
			case 27:
				this.cancel();
				break;
		}
		e.stopPropagation();
	}
	, onKeyUp: function (e) {
		e.stopPropagation();
	}
	, finish: function () {
		this.removeEventListener("focusOut", $bind(this, this.onFocusOut));
		this.removeEventListener("keyDown", $bind(this, this.onKeyDown));
		this.parent.removeChild(this);
	}
	, submit: function () {
		this.finish();
		if (this.onSubmit != null) {
			this.onSubmit(this.get_text());
		}
	}
	, cancel: function () {
		this.finish();
		if (this.onCancel != null) {
			this.onCancel();
		}
	}
	, __class__: com_watabou_mfcg_ui_EditInPlace
});
var com_watabou_mfcg_ui_Outline = function (obj, color, size, quality) {
	if (quality == null) {
		quality = 4.0;
	}
	this.matrix = new openfl_geom_Matrix();
	this.obj = obj;
	this.size = size;
	this.quality = quality;
	this.color = new openfl_geom_ColorTransform();
	this.color.set_color(color);
	openfl_display_Bitmap.call(this);
	this.update(quality);
};
$hxClasses["com.watabou.mfcg.ui.Outline"] = com_watabou_mfcg_ui_Outline;
com_watabou_mfcg_ui_Outline.__name__ = "com.watabou.mfcg.ui.Outline";
com_watabou_mfcg_ui_Outline.__super__ = openfl_display_Bitmap;
com_watabou_mfcg_ui_Outline.prototype = $extend(openfl_display_Bitmap.prototype, {
	update: function (scale) {
		if (scale == null) {
			scale = 0.0;
		}
		if (this.get_bitmapData() != null) {
			this.get_bitmapData().dispose();
		}
		if (scale == 0) {
			scale = this.quality;
		}
		var rect = this.obj.getRect(this.obj);
		var colored = new openfl_display_BitmapData(Math.ceil(rect.width * scale), Math.ceil(rect.height * scale), true, 0);
		this.matrix.identity();
		this.matrix.translate(-rect.get_left(), -rect.get_top());
		this.matrix.scale(scale, scale);
		colored.draw(this.obj, this.matrix, null, null, null, true);
		colored.colorTransform(colored.rect, this.color);
		var result = new openfl_display_BitmapData(Math.ceil((rect.width + this.size * 2) * scale), Math.ceil((rect.height + this.size * 2) * scale), true, 0);
		var nCopies = Math.ceil(scale * this.size * 6);
		var _g = 0;
		var _g1 = nCopies;
		while (_g < _g1) {
			var i = _g++;
			var ofs = openfl_geom_Point.polar(this.size, i / nCopies * 2 * Math.PI);
			ofs.offset(this.size, this.size);
			this.matrix.identity();
			this.matrix.translate(ofs.x * scale, ofs.y * scale);
			result.draw(colored, this.matrix, null, null, null, true);
		}
		colored.dispose();
		this.set_bitmapData(result);
		this.smoothing = true;
		this.set_scaleX(this.set_scaleY(1 / scale));
		this.set_x(this.obj.get_x() + rect.get_left() - this.size);
		this.set_y(this.obj.get_y() + rect.get_top() - this.size);
	}
	, __class__: com_watabou_mfcg_ui_Outline
});
var com_watabou_mfcg_ui_Text = function () { };
$hxClasses["com.watabou.mfcg.ui.Text"] = com_watabou_mfcg_ui_Text;
com_watabou_mfcg_ui_Text.__name__ = "com.watabou.mfcg.ui.Text";
com_watabou_mfcg_ui_Text.get = function (text, format) {
	var tf = new openfl_text_TextField();
	tf.set_defaultTextFormat(format);
	tf.set_autoSize(1);
	tf.set_text(text);
	return tf;
};
com_watabou_mfcg_ui_Text.embedded = function (font, size, color) {
	if (color == null) {
		color = 0;
	}
	return new openfl_text_TextFormat(openfl_utils_Assets.getFont(font).name, Math.round(size), color);
};
com_watabou_mfcg_ui_Text.getFormat = function (id, def, color, scale) {
	if (scale == null) {
		scale = 1.0;
	}
	if (id != null) {
		def = com_watabou_system_State.get(id, def);
	}
	var face = def.face != null ? def.face : openfl_utils_Assets.getFont(def.embedded).name;
	var size = Math.round(def.size * scale * com_watabou_mfcg_ui_Text.getMultiplier());
	return new openfl_text_TextFormat(face, size, color, def.bold, def.italic);
};
com_watabou_mfcg_ui_Text.getMultiplier = function () {
	switch (com_watabou_system_State.get("text_size", 1)) {
		case 0:
			return 0.50;
		case 1:
			return 0.75;
		default:
			return 1.00;
	}
};
var com_watabou_mfcg_ui_Tooltip = function () {
	this.isAwake = false;
	this.awake = new msignal_Signal1();
	com_watabou_mfcg_ui_Tooltip.instance = this;
	openfl_display_Sprite.call(this);
	this.border = new openfl_display_Bitmap(new openfl_display_BitmapData(1, 1, false, com_watabou_coogee_ui_UIStyle.black));
	this.addChild(this.border);
	this.bg = new openfl_display_Bitmap(new openfl_display_BitmapData(1, 1, false, com_watabou_coogee_ui_UIStyle.white));
	this.bg.set_x(1);
	this.bg.set_y(1);
	this.addChild(this.bg);
	var color = com_watabou_coogee_ui_UIStyle.black;
	if (color == null) {
		color = 0;
	}
	this.tf = com_watabou_mfcg_ui_Text.get("", new openfl_text_TextFormat(openfl_utils_Assets.getFont("ui_font").name, Math.round(18), color));
	this.tf.set_x(6);
	this.tf.set_y(6);
	this.addChild(this.tf);
	com_watabou_utils_DisplayObjectExtender.onActivate(this, $bind(this, this.activation));
	this.set(null);
};
$hxClasses["com.watabou.mfcg.ui.Tooltip"] = com_watabou_mfcg_ui_Tooltip;
com_watabou_mfcg_ui_Tooltip.__name__ = "com.watabou.mfcg.ui.Tooltip";
com_watabou_mfcg_ui_Tooltip.__super__ = openfl_display_Sprite;
com_watabou_mfcg_ui_Tooltip.prototype = $extend(openfl_display_Sprite.prototype, {
	activation: function (active) {
		if (active) {
			this.stage.addEventListener("mouseMove", $bind(this, this.onMouseMove));
			this.stage.addEventListener("mouseDown", $bind(this, this.onMouseMove));
			this.timer = com_watabou_utils_Updater.wait(5, $bind(this, this.fallAsleep));
		} else {
			this.stage.removeEventListener("mouseMove", $bind(this, this.onMouseMove));
			this.stage.removeEventListener("mouseDown", $bind(this, this.onMouseMove));
			if (this.timer != null) {
				com_watabou_utils_Updater.cancel(this.timer);
			}
		}
	}
	, onMouseMove: function (e) {
		this.set_x(this.parent.get_mouseX() + 16 / this.parent.get_scaleX());
		this.set_y(this.parent.get_mouseY());
		e.updateAfterEvent();
		this.wakeUp();
	}
	, fallAsleep: function () {
		this.awake.dispatch(this.isAwake = false);
		this.timer = null;
	}
	, wakeUp: function () {
		if (!this.isAwake) {
			this.awake.dispatch(this.isAwake = true);
		}
		if (this.timer != null) {
			com_watabou_utils_Updater.cancel(this.timer);
		}
		this.timer = com_watabou_utils_Updater.wait(5, $bind(this, this.fallAsleep));
	}
	, set: function (txt) {
		this.set_visible(txt != null);
		if (this.get_visible()) {
			this.tf.set_text(txt);
			var w = this.tf.get_width() + 12 | 0;
			var h = this.tf.get_height() + 12 | 0;
			this.border.set_width(w);
			this.border.set_height(h);
			this.bg.set_width(w - 2);
			this.bg.set_height(h - 2);
		}
	}
	, __class__: com_watabou_mfcg_ui_Tooltip
});
var com_watabou_mfcg_ui_forms_EmblemForm = function (emblem) {
	var _gthis = this;
	com_watabou_coogee_ui_ButtonsForm.call(this, ["OK", "Cancel"]);
	this.emblem = emblem;
	var link = com_watabou_mfcg_ui_forms_EmblemForm.EDITOR;
	if (com_watabou_mfcg_scenes_overlays_Emblem.coa != null) {
		link += "?coa=" + com_watabou_mfcg_scenes_overlays_Emblem.coa;
	}
	var content = new com_watabou_coogee_ui_layouts_VBox();
	content.setMargins(12, 10);
	content.add(new com_watabou_coogee_ui_Label("Open <a href=\"" + link + "\"><b>Armoria editor</b></a> and design your emblem.<br/><i>Copy COA String</i> there and paste it here."));
	this.input = new com_watabou_coogee_ui_TextInput();
	this.input.enter.add(function (txt) {
		_gthis.onEnter();
	});
	this.input.set_prompt("COA");
	this.input.halign = "fill";
	content.add(this.input);
	this.add(content);
	if (com_watabou_mfcg_scenes_overlays_Emblem.coa != null) {
		this.input.set_text(com_watabou_mfcg_scenes_overlays_Emblem.coa);
	}
};
$hxClasses["com.watabou.mfcg.ui.forms.EmblemForm"] = com_watabou_mfcg_ui_forms_EmblemForm;
com_watabou_mfcg_ui_forms_EmblemForm.__name__ = "com.watabou.mfcg.ui.forms.EmblemForm";
com_watabou_mfcg_ui_forms_EmblemForm.__super__ = com_watabou_coogee_ui_ButtonsForm;
com_watabou_mfcg_ui_forms_EmblemForm.prototype = $extend(com_watabou_coogee_ui_ButtonsForm.prototype, {
	getTitle: function () {
		return "Emblem";
	}
	, onShow: function () {
		this.input.selecteAll();
		this.stage.set_focus(this.input);
	}
	, onEnter: function () {
		this.onButton("OK");
	}
	, onButton: function (btn) {
		if (btn == "OK") {
			this.update();
		}
		com_watabou_coogee_ui_ButtonsForm.prototype.onButton.call(this, btn);
	}
	, update: function () {
		com_watabou_mfcg_scenes_overlays_Emblem.setCOA(this.input.get_text() != "" ? this.input.get_text() : null);
	}
	, __class__: com_watabou_mfcg_ui_forms_EmblemForm
});
var com_watabou_mfcg_ui_forms_MarkerForm = function (landmark, creating) {
	if (creating == null) {
		creating = true;
	}
	var _gthis = this;
	com_watabou_coogee_ui_ButtonsForm.call(this, ["OK", creating ? "Cancel" : "Delete"]);
	this.landmark = landmark;
	this.creating = creating;
	var row = new com_watabou_coogee_ui_layouts_HBox();
	var label = new com_watabou_coogee_ui_Label("Name");
	label.valign = "center";
	row.add(label);
	this.input = new com_watabou_coogee_ui_TextInput(landmark.name);
	this.input.set_width(200);
	this.input.enter.add(function (name) {
		_gthis.onButton("OK");
	});
	row.add(this.input);
	this.add(row);
};
$hxClasses["com.watabou.mfcg.ui.forms.MarkerForm"] = com_watabou_mfcg_ui_forms_MarkerForm;
com_watabou_mfcg_ui_forms_MarkerForm.__name__ = "com.watabou.mfcg.ui.forms.MarkerForm";
com_watabou_mfcg_ui_forms_MarkerForm.__super__ = com_watabou_coogee_ui_ButtonsForm;
com_watabou_mfcg_ui_forms_MarkerForm.prototype = $extend(com_watabou_coogee_ui_ButtonsForm.prototype, {
	getTitle: function () {
		return "Landmark";
	}
	, onShow: function () {
		this.stage.set_focus(this.input);
	}
	, onButton: function (btn) {
		var model = com_watabou_mfcg_model_City.instance;
		switch (btn) {
			case "Cancel":
				if (this.creating) {
					model.removeLandmark(this.landmark);
				}
				break;
			case "Delete":
				model.removeLandmark(this.landmark);
				break;
			case "OK":
				this.landmark.name = this.input.get_text();
				com_watabou_mfcg_model_ModelDispatcher.landmarksChanged.dispatch();
				break;
		}
		this.dialog.hide();
	}
	, __class__: com_watabou_mfcg_ui_forms_MarkerForm
});
var com_watabou_mfcg_ui_forms_TownInfo = function (model) {
	com_watabou_coogee_ui_View.call(this);
	var f = com_watabou_coogee_ui_UIStyle.format(com_watabou_coogee_ui_UIStyle.uiFont, com_watabou_coogee_ui_UIStyle.smallSize, com_watabou_coogee_ui_UIStyle.black);
	f.align = 0;
	this.tf = com_watabou_coogee_ui_utils_Text.get("", f);
	this.tf.set_x(-2);
	this.tf.set_y(-2);
	this.addChild(this.tf);
	this.update(model);
};
$hxClasses["com.watabou.mfcg.ui.forms.TownInfo"] = com_watabou_mfcg_ui_forms_TownInfo;
com_watabou_mfcg_ui_forms_TownInfo.__name__ = "com.watabou.mfcg.ui.forms.TownInfo";
com_watabou_mfcg_ui_forms_TownInfo.__super__ = com_watabou_coogee_ui_View;
com_watabou_mfcg_ui_forms_TownInfo.prototype = $extend(com_watabou_coogee_ui_View.prototype, {
	update: function (model) {
		var n = model.countBuildings();
		var p = model.bp.pop;
		if (p == 0) {
			p = n * 6;
		}
		var d = Math.pow(10, Math.floor(Math.log(p) / Math.log(10)) - 1);
		p = Math.ceil(p / d) * d | 0;
		this.tf.set_autoSize(1);
		this.tf.set_text("Number of buildings: " + n + "\n" + ("Population: ~" + p));
		var w = this.tf.get_width();
		var h = this.tf.get_height();
		this.tf.set_autoSize(2);
		this.tf.set_width(w);
		this.tf.set_height(Math.ceil(h));
		this.setSize(this.tf.get_width() - 4, this.tf.get_height() - 4 - 1);
	}
	, __class__: com_watabou_mfcg_ui_forms_TownInfo
});
var com_watabou_mfcg_ui_forms_URLForm = function () {
	var _gthis = this;
	var btns = ["Copy", "Generate"];
	com_watabou_coogee_ui_ButtonsForm.call(this, btns);
	var content = new com_watabou_coogee_ui_layouts_VBox();
	var info = new com_watabou_coogee_ui_layouts_VBox();
	info.setMargins(0, 0);
	info.add(new com_watabou_coogee_ui_Label("Copy this URL to restore the current map later."));
	info.add(new com_watabou_coogee_ui_Label("Enter a stored URL here to restore a map."));
	content.add(info);
	this.input = new com_watabou_coogee_ui_TextInput(com_watabou_system_URLState.getURL());
	this.input.enter.add(function (txt) {
		_gthis.onEnter();
	});
	this.input.set_prompt("URL");
	this.input.set_width(400);
	content.add(this.input);
	this.add(content);
	com_watabou_mfcg_model_ModelDispatcher.newModel.add($bind(this, this.onNewModel));
};
$hxClasses["com.watabou.mfcg.ui.forms.URLForm"] = com_watabou_mfcg_ui_forms_URLForm;
com_watabou_mfcg_ui_forms_URLForm.__name__ = "com.watabou.mfcg.ui.forms.URLForm";
com_watabou_mfcg_ui_forms_URLForm.__super__ = com_watabou_coogee_ui_ButtonsForm;
com_watabou_mfcg_ui_forms_URLForm.prototype = $extend(com_watabou_coogee_ui_ButtonsForm.prototype, {
	getTitle: function () {
		return "Permalink";
	}
	, onShow: function () {
		this.highlight();
	}
	, onHide: function () {
		com_watabou_mfcg_model_ModelDispatcher.newModel.remove($bind(this, this.onNewModel));
	}
	, onEnter: function () {
		this.onButton("Generate");
	}
	, onButton: function (btn) {
		switch (btn) {
			case "Copy":
				com_watabou_mfcg_Buffer.write(com_watabou_system_URLState.getURL());
				com_watabou_coogee_ui_Toast.show("URL was copied to clipboard");
				this.highlight();
				break;
			case "Generate":
				this.generate();
				this.highlight();
				break;
			default:
				com_watabou_coogee_ui_ButtonsForm.prototype.onButton.call(this, btn);
		}
	}
	, onNewModel: function (model) {
		this.input.set_text(com_watabou_system_URLState.getURL());
		this.highlight();
	}
	, generate: function () {
		var url = this.input.get_text();
		com_watabou_system_URLState.fromString(url);
		new com_watabou_mfcg_model_City(com_watabou_mfcg_model_Blueprint.fromURL());
		com_watabou_coogee_Game.switchScene(com_watabou_mfcg_scenes_ViewScene);
		var form = com_watabou_coogee_ui_UI.findForm(com_watabou_mfcg_ui_forms_GenerateForm);
		if (form != null) {
			form.update();
		}
	}
	, highlight: function () {
		this.stage.set_focus(this.input);
		this.input.selecteAll();
	}
	, __class__: com_watabou_mfcg_ui_forms_URLForm
});
var com_watabou_mfcg_utils_Bisector = function (poly, minArea, variance) {
	if (variance == null) {
		variance = 10.0;
	}
	this.cuts = [];
	this.minTurnOffset = 1.0;
	this.poly = poly;
	this.minArea = minArea;
	this.variance = variance;
	this.minOffset = Math.sqrt(minArea);
	this.processCut = $bind(this, this.detectStraight);
	this.shape = poly;
};
$hxClasses["com.watabou.mfcg.utils.Bisector"] = com_watabou_mfcg_utils_Bisector;
com_watabou_mfcg_utils_Bisector.__name__ = "com.watabou.mfcg.utils.Bisector";
com_watabou_mfcg_utils_Bisector.prototype = {
	partition: function () {
		return this.subdivide(this.shape);
	}
	, subdivide: function (poly) {
		var minArea = this.getMinArea != null ? this.getMinArea(poly) : this.minArea * Math.pow(this.variance, Math.abs(((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 2 - 1));
		var area = com_watabou_geom_polygons_PolyCore.area(poly);
		if (area < minArea) {
			return [poly];
		}
		var parts = [];
		var halves = this.makeCut(poly);
		var _g = 0;
		while (_g < halves.length) {
			var half = halves[_g];
			++_g;
			var b = this.subdivide(half);
			var _g1 = 0;
			while (_g1 < b.length) {
				var e = b[_g1];
				++_g1;
				parts.push(e);
			}
		}
		return parts;
	}
	, varMinArea: function () {
		return this.minArea * Math.pow(this.variance, Math.abs(((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 2 - 1));
	}
	, makeCut: function (poly, alt) {
		if (alt == null) {
			alt = false;
		}
		var len = poly.length;
		var rect;
		if (alt) {
			var a = openfl_geom_Point.polar(1, 2 * Math.PI * ((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647));
			var poly1 = com_watabou_geom_polygons_PolyTransform.rotateYX(poly, a.y, a.x);
			rect = com_watabou_geom_polygons_PolyTransform.rotateYX(com_watabou_geom_polygons_PolyBounds.aabb(poly1), -a.y, a.x);
		} else {
			rect = com_watabou_geom_polygons_PolyBounds.obb(poly);
		}
		var origin = rect[0];
		var axis = rect[1].subtract(origin);
		var cutDir = rect[3].subtract(origin);
		if (axis.get_length() < cutDir.get_length()) {
			var t = axis;
			axis = cutDir;
			cutDir = t;
		}
		var ofs2axis = this.minOffset / axis.get_length();
		var ratio = ofs2axis > 0.5 ? 0.5 : ofs2axis + (1 - 2 * ofs2axis) * (((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 3);
		var cutStart = new openfl_geom_Point(origin.x + axis.x * ratio, origin.y + axis.y * ratio);
		var start = null;
		var dir = null;
		var startEdge = -1;
		var maxColinearity = 0.0;
		var _g = 0;
		var _g1 = len;
		while (_g < _g1) {
			var i = _g++;
			var p0 = poly[i];
			var p1 = poly[(i + 1) % len];
			var d = p1.subtract(p0);
			if (d.get_length() < 1e-10) {
				continue;
			}
			var t = com_watabou_geom_GeomUtils.intersectLines(cutStart.x, cutStart.y, cutDir.x, cutDir.y, p0.x, p0.y, d.x, d.y);
			if (t != null && t.y > 0 && t.y < 1) {
				var p = d;
				p = p.clone();
				p.normalize(1);
				var dNorm = p;
				var colinearity = Math.abs(axis.x * dNorm.x + axis.y * dNorm.y);
				if (maxColinearity < colinearity) {
					maxColinearity = colinearity;
					startEdge = i;
					var t1 = t.y;
					start = new openfl_geom_Point(p0.x + d.x * t1, p0.y + d.y * t1);
					dir = dNorm;
				}
			}
		}
		dir.setTo(-dir.y, dir.x);
		var distance = Infinity;
		var otherDir = null;
		var _g = 0;
		var _g1 = len;
		while (_g < _g1) {
			var i = _g++;
			if (i != startEdge) {
				var p0 = poly[i];
				var p1 = poly[(i + 1) % len];
				var d = p1.subtract(p0);
				if (d.get_length() < 1e-10) {
					continue;
				}
				var j = com_watabou_geom_GeomUtils.intersectLines(start.x, start.y, dir.x, dir.y, p0.x, p0.y, d.x, d.y);
				if (j != null && j.x > 0 && j.x < distance && j.y > 0 && j.y < 1) {
					distance = j.x;
					otherDir = d;
				}
			}
		}
		if (otherDir == null) {
			haxe_Log.trace("recut - a bad poly was provided", { fileName: "Source/com/watabou/mfcg/utils/Bisector.hx", lineNumber: 150, className: "com.watabou.mfcg.utils.Bisector", methodName: "makeCut" });
			return this.makeCut(poly, true);
		}
		var norm = true;
		if (norm == null) {
			norm = false;
		}
		otherDir.setTo(otherDir.y, -otherDir.x);
		if (norm) {
			otherDir.normalize(1);
		}
		var ofs2dist = this.minOffset / distance;
		var ratio = ofs2dist > 0.5 ? 0.5 : ofs2dist + (1 - 2 * ofs2dist) * (((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 3);
		var t = distance * ratio;
		var turn = new openfl_geom_Point(start.x + dir.x * t, start.y + dir.y * t);
		var end = null;
		var endEdge = -1;
		var distance = -Infinity;
		var _g = 0;
		var _g1 = len;
		while (_g < _g1) {
			var i = _g++;
			if (i != startEdge) {
				var p0 = poly[i];
				var p1 = poly[(i + 1) % len];
				var d = p1.subtract(p0);
				var dLen = d.get_length();
				if (dLen < 1e-10) {
					continue;
				}
				var t = com_watabou_geom_GeomUtils.intersectLines(turn.x, turn.y, d.y, -d.x, p0.x, p0.y, d.x, d.y);
				if (t.x > 0 && t.y > 0 && t.y < 1) {
					var cos = (dir.x * d.y - dir.y * d.x) / dLen;
					var dist = cos;
					if (distance < dist) {
						var valid = true;
						var _g2 = 0;
						var _g3 = len;
						while (_g2 < _g3) {
							var j = _g2++;
							if (j != i && j != startEdge) {
								var q0 = poly[j];
								var q1 = poly[(j + 1) % len];
								var e = q1.subtract(q0);
								if (e.get_length() < 1e-10) {
									continue;
								}
								var t1 = com_watabou_geom_GeomUtils.intersectLines(turn.x, turn.y, d.y, -d.x, q0.x, q0.y, e.x, e.y);
								if (t1 != null && t1.x >= 0 && t1.x <= 1 && t1.y >= 0 && t1.y <= 1) {
									valid = false;
									break;
								}
							}
						}
						if (valid) {
							distance = dist;
							endEdge = i;
							var t2 = t.y;
							end = new openfl_geom_Point(p0.x + d.x * t2, p0.y + d.y * t2);
						}
					}
				}
			}
		}
		if (end != null) {
			var points = this.processCut([start, turn, end]);
			var halves = this.split(poly, startEdge, endEdge, points);
			var a1 = com_watabou_geom_polygons_PolyCore.area(halves[0]);
			var a2 = com_watabou_geom_polygons_PolyCore.area(halves[1]);
			if (Math.max(a1, a2) / Math.min(a1, a2) > this.variance) {
				return this.makeCut(poly, true);
			} else {
				this.cuts.push(points);
				if (this.getGap != null) {
					var gapShape = com_watabou_geom_polygons_PolyCreate.stripe(points, this.getGap(points));
					var _g = [];
					var _g1 = 0;
					while (_g1 < halves.length) {
						var half = halves[_g1];
						++_g1;
						var result = com_watabou_geom_polygons_PolyBool.and(half, com_watabou_utils_ArrayExtender.revert(gapShape), true);
						_g.push(result != null ? result : half);
					}
					halves = _g;
				}
				return halves;
			}
		} else {
			haxe_Log.trace("Failed to make a cut, trying again...", { fileName: "Source/com/watabou/mfcg/utils/Bisector.hx", lineNumber: 226, className: "com.watabou.mfcg.utils.Bisector", methodName: "makeCut" });
			return this.makeCut(poly, true);
		}
	}
	, split: function (poly, i0, i1, points) {
		var e0 = poly[i0];
		var e1 = poly[i1];
		var first = points[0];
		if (e0 != first) {
			if (i0 < i1) {
				++i1;
			}
			poly.splice(++i0, 0, first);
		}
		var last = points[points.length - 1];
		if (e1 != last) {
			if (i1 < i0) {
				++i0;
			}
			poly.splice(++i1, 0, last);
		}
		var half0;
		var half1;
		if (i0 < i1) {
			half0 = poly.slice(i0 + 1, i1);
			var b = com_watabou_utils_ArrayExtender.revert(points);
			var _g = 0;
			while (_g < b.length) {
				var e = b[_g];
				++_g;
				half0.push(e);
			}
			half1 = poly.slice(i1 + 1);
			var b = poly.slice(0, i0);
			var _g = 0;
			while (_g < b.length) {
				var e = b[_g];
				++_g;
				half1.push(e);
			}
			var _g = 0;
			while (_g < points.length) {
				var e = points[_g];
				++_g;
				half1.push(e);
			}
		} else {
			half0 = poly.slice(i0 + 1);
			var b = poly.slice(0, i1);
			var _g = 0;
			while (_g < b.length) {
				var e = b[_g];
				++_g;
				half0.push(e);
			}
			var b = com_watabou_utils_ArrayExtender.revert(points);
			var _g = 0;
			while (_g < b.length) {
				var e = b[_g];
				++_g;
				half0.push(e);
			}
			half1 = poly.slice(i1 + 1, i0);
			var _g = 0;
			while (_g < points.length) {
				var e = points[_g];
				++_g;
				half1.push(e);
			}
		}
		return [half0, half1];
	}
	, detectStraight: function (cut) {
		if (this.minTurnOffset > 0) {
			var start = cut[0];
			var end = cut[2];
			var triArea = Math.abs(com_watabou_geom_polygons_PolyCore.area(cut));
			if (triArea / openfl_geom_Point.distance(start, end) < this.minTurnOffset) {
				return [start, end];
			} else {
				return cut;
			}
		} else {
			return cut;
		}
	}
	, smoothTurn: function (cut) {
		cut = this.detectStraight(cut);
		if (cut.length > 2) {
			return com_watabou_geom_Chaikin.render(cut, false, 3);
		} else {
			return cut;
		}
	}
	, __class__: com_watabou_mfcg_utils_Bisector
};
var com_watabou_mfcg_utils_Cutter = function () { };
$hxClasses["com.watabou.mfcg.utils.Cutter"] = com_watabou_mfcg_utils_Cutter;
com_watabou_mfcg_utils_Cutter.__name__ = "com.watabou.mfcg.utils.Cutter";
com_watabou_mfcg_utils_Cutter.curl = function (poly, p1, p2, swing, gap) {
	if (gap == null) {
		gap = 0.0;
	}
	if (swing == null) {
		swing = 1.0;
	}
	var halves = com_watabou_geom_polygons_PolyCut.cut(poly, p1, p2);
	var q_0 = halves[0][0];
	var a = halves[0];
	var q_1 = a[a.length - 1];
	var len = openfl_geom_Point.distance(q_0, q_1) * swing * (((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 3 - 0.5);
	var middle = com_watabou_geom_GeomUtils.lerp(q_0, q_1).add(openfl_geom_Point.polar(len, (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 * Math.PI));
	halves[0].unshift(middle);
	halves[1].unshift(middle);
	var convex = com_watabou_geom_polygons_PolyAccess.isConvexVertexi(halves[0], 0) ? 0 : 1;
	var len = halves[convex].length;
	var halves1 = halves[convex];
	var _g = [];
	var _g1 = 0;
	var _g2 = len;
	while (_g1 < _g2) {
		var i = _g1++;
		_g.push(i == 0 || i == len - 1 ? gap : 0.0);
	}
	halves[convex] = com_watabou_geom_polygons_PolyCut.shrink(halves1, _g);
	return halves;
};
com_watabou_mfcg_utils_Cutter.radial = function (poly, center, gap) {
	if (gap == null) {
		gap = 0.0;
	}
	if (center == null) {
		center = com_watabou_geom_polygons_PolyCore.centroid(poly);
	}
	var len = poly.length;
	gap /= 2;
	var sectors = [];
	var _g = 0;
	var _g1 = len;
	while (_g < _g1) {
		var i = _g++;
		var v0 = poly[i];
		var v1 = poly[(i + 1) % len];
		var sector = [center, v0, v1];
		if (gap > 0) {
			sector = com_watabou_geom_polygons_PolyCut.shrink(sector, [gap, 0, gap]);
		}
		sectors.push(sector);
	}
	return sectors;
};
com_watabou_mfcg_utils_Cutter.semiRadial = function (poly, center, gap) {
	if (gap == null) {
		gap = 0.0;
	}
	if (center == null) {
		var centroid = com_watabou_geom_polygons_PolyCore.centroid(poly);
		center = com_watabou_utils_ArrayExtender.min(poly, function (v) {
			return openfl_geom_Point.distance(v, centroid);
		});
	}
	var len = poly.length;
	gap /= 2;
	var sectors = [];
	var _g = 0;
	var _g1 = len;
	while (_g < _g1) {
		var i = _g++;
		var v0 = poly[i];
		var v1 = poly[(i + 1) % len];
		if (v0 != center && v1 != center) {
			var sector = [center, v0, v1];
			if (gap > 0) {
				var d = [poly[(i + len - 1) % len] == center ? 0 : gap, 0, poly[(i + 2) % len] == center ? 0 : gap];
				sector = com_watabou_geom_polygons_PolyCut.shrink(sector, d);
			}
			sectors.push(sector);
		}
	}
	return sectors;
};
com_watabou_mfcg_utils_Cutter.grid = function (poly, cols, rows, chaos) {
	if (chaos == null) {
		chaos = 0.0;
	}
	if (poly.length != 4) {
		throw new openfl_errors_Error("Not a quadrangle!");
	}
	var _g = [];
	var _g1 = 0;
	var _g2 = cols + 1;
	while (_g1 < _g2) {
		var i = _g1++;
		_g.push(i / cols);
	}
	var hratios = _g;
	var _g = [];
	var _g1 = 0;
	var _g2 = rows + 1;
	while (_g1 < _g2) {
		var i = _g1++;
		_g.push(i / rows);
	}
	var vratios = _g;
	if (chaos > 0) {
		var _g = 1;
		var _g1 = cols;
		while (_g < _g1) {
			var i = _g++;
			hratios[i] += (((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 3 - 0.5) / (cols - 1) * chaos;
		}
		var _g = 1;
		var _g1 = rows;
		while (_g < _g1) {
			var i = _g++;
			vratios[i] += (((com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647 + (com_watabou_utils_Random.seed = com_watabou_utils_Random.seed * 48271.0 % 2147483647 | 0) / 2147483647) / 3 - 0.5) / (rows - 1) * chaos;
		}
	}
	var p0 = poly[0];
	var p1 = poly[1];
	var p2 = poly[2];
	var p3 = poly[3];
	var _g = [];
	var _g1 = 0;
	var _g2 = rows + 1;
	while (_g1 < _g2) {
		var i = _g1++;
		var left = com_watabou_geom_GeomUtils.lerp(p0, p3, vratios[i]);
		var right = com_watabou_geom_GeomUtils.lerp(p1, p2, vratios[i]);
		var _g3 = [];
		var _g4 = 0;
		var _g5 = cols + 1;
		while (_g4 < _g5) {
			var j = _g4++;
			_g3.push(com_watabou_geom_GeomUtils.lerp(left, right, hratios[j]));
		}
		_g.push(_g3);
	}
	var points = _g;
	var cells = [];
	var _g = 0;
	var _g1 = rows;
	while (_g < _g1) {
		var i = _g++;
		var _g2 = 0;
		var _g3 = cols;
		while (_g2 < _g3) {
			var j = _g2++;
			cells.push([points[i][j], points[i][j + 1], points[i + 1][j + 1], points[i + 1][j]]);
		}
	}
	return cells;
};
var com_watabou_mfcg_utils_PolyUtils = function () { };
$hxClasses["com.watabou.mfcg.utils.PolyUtils"] = com_watabou_mfcg_utils_PolyUtils;
com_watabou_mfcg_utils_PolyUtils.__name__ = "com.watabou.mfcg.utils.PolyUtils";
com_watabou_mfcg_utils_PolyUtils.lerpVertex = function (poly, p) {
	var i = poly.indexOf(p);
	var len = poly.length;
	var prev = poly[(i + len - 1) % len];
	var next = poly[(i + 1) % len];
	return com_watabou_geom_GeomUtils.lerp(prev, next);
};
com_watabou_mfcg_utils_PolyUtils.smooth = function (poly, reserved, power) {
	if (power == null) {
		power = 1;
	}
	var len = poly.length;
	var _g = 0;
	var _g1 = power;
	while (_g < _g1) {
		var _ = _g++;
		var _g2 = [];
		var _g3 = 0;
		var _g4 = len;
		while (_g3 < _g4) {
			var i = _g3++;
			var v1 = poly[i];
			if (reserved != null && reserved.indexOf(v1) != -1) {
				_g2.push(v1);
			} else {
				var v0 = poly[(i + len - 1) % len];
				var v2 = poly[(i + 1) % len];
				_g2.push(com_watabou_geom_GeomUtils.lerp(com_watabou_geom_GeomUtils.lerp(v0, v2), v1));
			}
		}
		var smooth = _g2;
		poly = smooth;
	}
	return poly;
};
com_watabou_mfcg_utils_PolyUtils.smoothOpen = function (poly, reserved, power) {
	if (power == null) {
		power = 1;
	}
	var len = poly.length;
	var _g = 0;
	var _g1 = power;
	while (_g < _g1) {
		var _ = _g++;
		var _g2 = [];
		var _g3 = 0;
		var _g4 = len;
		while (_g3 < _g4) {
			var i = _g3++;
			var v1 = poly[i];
			if (i == 0 || i == len - 1 || reserved != null && reserved.indexOf(v1) != -1) {
				_g2.push(v1);
			} else {
				var v0 = poly[i - 1];
				var v2 = poly[i + 1];
				_g2.push(com_watabou_geom_GeomUtils.lerp(com_watabou_geom_GeomUtils.lerp(v0, v2), v1));
			}
		}
		var smooth = _g2;
		poly = smooth;
	}
	return poly;
};
com_watabou_mfcg_utils_PolyUtils.simpleInset = function (poly, dist) {
	var n = poly.length;
	var _g = [];
	var _g1 = 0;
	var _g2 = n;
	while (_g1 < _g2) {
		var i = _g1++;
		var p0 = poly[(i + n - 1) % n];
		var p1 = poly[i];
		var p2 = poly[(i + 1) % n];
		var d0 = dist[(i + n - 1) % n];
		var d1 = dist[i];
		var v1 = p1.subtract(p0);
		var l1 = v1.get_length();
		var v2 = p2.subtract(p1);
		var l2 = v2.get_length();
		var sin = v1.x * v2.y - v1.y * v2.x;
		var t = -d1 / (sin / l2);
		var a = new openfl_geom_Point(p1.x + v1.x * t, p1.y + v1.y * t);
		var t1 = d0 / (sin / l1);
		_g.push(new openfl_geom_Point(a.x + v2.x * t1, a.y + v2.y * t1));
	}
	return _g;
};
com_watabou_mfcg_utils_PolyUtils.inset = function (poly, dist) {
	var result = poly;
	var len = poly.length;
	var start = 0;
	var _g = 0;
	var _g1 = len;
	while (_g < _g1) {
		var i = _g++;
		if (dist[i] != dist[(i + len - 1) % len]) {
			start = i;
			break;
		}
	}
	var a = start;
	var e = [poly[a]];
	var d = dist[a];
	while (true) {
		var b = a;
		while (true) {
			b = (b + 1) % len;
			e.push(poly[b]);
			if (!(b != start && dist[b] == d)) {
				break;
			}
		}
		if (d != 0.0) {
			var stripe = com_watabou_geom_polygons_PolyCreate.stripe(e, d * 2);
			var result1 = com_watabou_geom_polygons_PolyBool.and(result, com_watabou_utils_ArrayExtender.revert(stripe), true);
			if (result1 != null) {
				result = result1;
			}
		}
		if (b == start) {
			break;
		} else {
			a = b;
			e = [poly[a]];
			d = dist[a];
		}
	}
	return result;
};