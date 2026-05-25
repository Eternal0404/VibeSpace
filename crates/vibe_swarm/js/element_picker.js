/**
 * VibeTerminal Web Preview DOM Element Picker
 * 
 * This script is injected into the web preview iframe to enable
 * element selection and "vibe coding" functionality.
 * 
 * Features:
 * - Mouse hover highlighting with visual outline
 * - Click to select element and capture metadata
 * - CSS selector path generation
 * - Communication with host via window.parent.postMessage
 */

(function() {
    'use strict';

    const CONFIG = {
        highlightColor: '#00D4FF',
        highlightBorderWidth: 2,
        highlightZIndex: 999999,
        debugMode: false,
        maxSelectorDepth: 10,
        maxXPathDepth: 20,
        maxDataAttrs: 20,
        allowedOrigin: '*',
        minSimilarityThreshold: 0.7
    };

    let isSelecting = false;
    let currentHighlight = null;
    let lastHoveredElement = null;
    let selectionCallback = null;
    let messageHandler = null;

    function log(...args) {
        if (CONFIG.debugMode) {
            console.log('[ElementPicker]', ...args);
        }
    }

    function createHighlightOverlay() {
        const overlay = document.createElement('div');
        overlay.id = 'vibe-element-highlight';
        overlay.style.cssText = [
            'position: fixed',
            'pointer-events: none',
            'z-index: ' + CONFIG.highlightZIndex,
            'background-color: transparent',
            'border: ' + CONFIG.highlightBorderWidth + 'px solid ' + CONFIG.highlightColor,
            'box-sizing: border-box',
            'transition: all 0.1s ease-out',
            'border-radius: 2px',
            'box-shadow: 0 0 8px ' + CONFIG.highlightColor + '80'
        ].join(';');
        document.body.appendChild(overlay);
        return overlay;
    }

    function getElementRect(element) {
        const rect = element.getBoundingClientRect();
        return {
            top: rect.top + window.scrollY,
            left: rect.left + window.scrollX,
            width: rect.width,
            height: rect.height,
            right: rect.right + window.scrollX,
            bottom: rect.bottom + window.scrollY
        };
    }

    function updateHighlight(element) {
        if (!currentHighlight) {
            currentHighlight = createHighlightOverlay();
        }

        const rect = getElementRect(element);
        currentHighlight.style.top = rect.top + 'px';
        currentHighlight.style.left = rect.left + 'px';
        currentHighlight.style.width = rect.width + 'px';
        currentHighlight.style.height = rect.height + 'px';
        currentHighlight.style.display = 'block';
    }

    function hideHighlight() {
        if (currentHighlight) {
            currentHighlight.style.display = 'none';
        }
    }

    function cssEscape(str) {
        if (!str || str.length === 0) {
            return '';
        }
        var cssEsc = '';
        for (var i = 0; i < str.length; i++) {
            var char = str.charAt(i);
            var code = str.charCodeAt(i);
            if (char === '-' && i === 0) {
                cssEsc += '\\-';
            } else if (code >= 0x21 && code <= 0x7E && '!"#$%&\'()*+,./:;<=>?@[\\]^`{|}~'.indexOf(char) === -1) {
                cssEsc += char;
            } else {
                cssEsc += '\\' + char;
            }
        }
        return cssEsc;
    }

    function generateCSSSelector(element) {
        if (!element || element === document.body) {
            return 'body';
        }

        if (element.id) {
            return '#' + cssEscape(element.id);
        }

        var path = [];
        var current = element;
        var depth = 0;

        while (current && current !== document.body && depth < CONFIG.maxSelectorDepth) {
            var selector = '';

            if (current.id) {
                selector = '#' + cssEscape(current.id);
                path.unshift(selector);
                break;
            }

            if (current.className && typeof current.className === 'string' && current.className.trim()) {
                var classes = current.className.trim().split(/\s+/).slice(0, 2);
                if (classes.length > 0 && classes[0]) {
                    var escapedClasses = classes.map(function(c) { return cssEscape(c); }).join('.');
                    selector = current.tagName.toLowerCase() + '.' + escapedClasses;
                }
            }

            if (!selector) {
                selector = current.tagName.toLowerCase();
            }

            var siblings = current.parentNode ? 
                Array.prototype.slice.call(current.parentNode.children).filter(function(el) { return el.tagName === current.tagName; }) : [];
            
            if (siblings.length > 1) {
                var index = siblings.indexOf(current) + 1;
                selector += ':nth-child(' + index + ')';
            }

            path.unshift(selector);
            current = current.parentNode;
            depth++;
        }

        return path.join(' > ');
    }

    function generateXPath(element) {
        if (!element || element === document.body) {
            return '/html/body';
        }

        if (element.id) {
            var escapedId = element.id.replace(/"/g, '\\"');
            return '//*[@id="' + escapedId + '"]';
        }

        var parts = [];
        var current = element;
        var depth = 0;

        while (current && current !== document.body && current.nodeType === Node.ELEMENT_NODE && depth < CONFIG.maxXPathDepth) {
            var index = 1;
            var sibling = current.previousElementSibling;

            while (sibling) {
                if (sibling.tagName === current.tagName) {
                    index++;
                }
                sibling = sibling.previousElementSibling;
            }

            var tagName = current.tagName.toLowerCase();
            parts.unshift(tagName + '[' + index + ']');
            current = current.parentNode;
            depth++;
        }

        return '/html/' + parts.join('/');
    }

    function getElementMetadata(element) {
        if (!element || !(element instanceof HTMLElement)) {
            return null;
        }

        var rect = getElementRect(element);

        var computedStyle = window.getComputedStyle(element);
        
        var attributes = {};
        var dataAttrCount = 0;
        var attrWhitelist = ['role', 'aria-label', 'title', 'name', 'type', 'value'];
        
        for (var i = 0; i < element.attributes.length; i++) {
            var attr = element.attributes[i];
            if (attr.name.startsWith('data-') && dataAttrCount < CONFIG.maxDataAttrs) {
                attributes[attr.name] = attr.value;
                dataAttrCount++;
            } else if (attrWhitelist.indexOf(attr.name) !== -1) {
                attributes[attr.name] = attr.value;
            }
        }

        var innerText = '';
        if (element.innerText) {
            innerText = element.innerText.substring(0, 200);
        }
        
        var outerHTML = '';
        if (element.outerHTML) {
            outerHTML = element.outerHTML.substring(0, 500);
        }

        return {
            tagName: element.tagName ? element.tagName.toLowerCase() : null,
            id: element.id || null,
            className: element.className || null,
            classes: element.className ? Array.prototype.slice.call(element.classList).filter(function(c) { return c.trim(); }) : [],
            attributes: attributes,
            text: innerText,
            href: element.href || null,
            src: element.src || null,
            rect: rect,
            computedStyles: {
                display: computedStyle.display,
                position: computedStyle.position,
                width: computedStyle.width,
                height: computedStyle.height,
                color: computedStyle.color,
                backgroundColor: computedStyle.backgroundColor,
                fontSize: computedStyle.fontSize,
                fontFamily: computedStyle.fontFamily
            },
            cssSelector: generateCSSSelector(element),
            xpath: generateXPath(element),
            outerHTML: outerHTML,
            isInteractive: ['A', 'BUTTON', 'INPUT', 'SELECT', 'TEXTAREA'].indexOf(element.tagName) !== -1 ||
                          element.onclick !== null ||
                          element.getAttribute('role') === 'button' ||
                          computedStyle.cursor === 'pointer',
            isEditable: element.isContentEditable ||
                        ['INPUT', 'TEXTAREA', 'SELECT'].indexOf(element.tagName) !== -1
        };
    }

    function handleMouseOver(event) {
        if (!isSelecting) return;

        var target = event.target;
        if (!target || target === currentHighlight || (target.id && target.id === 'vibe-element-highlight')) {
            return;
        }

        lastHoveredElement = target;
        updateHighlight(target);
        
        log('Hovering:', target.tagName, target.className);
    }

    function handleMouseOut(event) {
        if (!isSelecting) return;
        
        var relatedTarget = event.relatedTarget;
        if (relatedTarget && relatedTarget !== document.body) {
            return;
        }
        
        hideHighlight();
        lastHoveredElement = null;
    }

    function handleClick(event) {
        if (!isSelecting) return;

        event.preventDefault();
        event.stopPropagation();

        var target = event.target;
        var metadata = getElementMetadata(target);

        if (!metadata) {
            log('No metadata for clicked element');
            return;
        }

        var message = {
            type: 'ELEMENT_SELECTED',
            payload: {
                metadata: metadata,
                timestamp: Date.now(),
                source: window.location.href
            }
        };

        log('Element selected:', metadata.tagName, metadata.cssSelector);
        
        try {
            window.parent.postMessage(message, CONFIG.allowedOrigin);
        } catch (error) {
            console.error('[ElementPicker] Failed to send message:', error);
        }

        hideHighlight();
    }

    function handleKeyDown(event) {
        if (!isSelecting) return;

        if (event.key === 'Escape') {
            stopSelection();
            try {
                window.parent.postMessage({
                    type: 'ELEMENT_SELECTION_CANCELLED',
                    payload: { timestamp: Date.now() }
                }, CONFIG.allowedOrigin);
            } catch (error) {
                console.error('[ElementPicker] Failed to send message:', error);
            }
        }
    }

    function startSelection(callback) {
        if (isSelecting) return;

        isSelecting = true;
        selectionCallback = callback || null;

        document.addEventListener('mouseover', handleMouseOver, true);
        document.addEventListener('mouseout', handleMouseOut, true);
        document.addEventListener('click', handleClick, true);
        document.addEventListener('keydown', handleKeyDown, true);

        document.body.style.cursor = 'crosshair';

        log('Element selection started');

        try {
            window.parent.postMessage({
                type: 'ELEMENT_SELECTION_STARTED',
                payload: { timestamp: Date.now() }
            }, CONFIG.allowedOrigin);
        } catch (error) {
            console.error('[ElementPicker] Failed to send message:', error);
        }
    }

    function stopSelection() {
        if (!isSelecting) return;

        isSelecting = false;

        document.removeEventListener('mouseover', handleMouseOver, true);
        document.removeEventListener('mouseout', handleMouseOut, true);
        document.removeEventListener('click', handleClick, true);
        document.removeEventListener('keydown', handleKeyDown, true);

        document.body.style.cursor = '';

        if (currentHighlight) {
            currentHighlight.parentNode && currentHighlight.parentNode.removeChild(currentHighlight);
            currentHighlight = null;
        }

        lastHoveredElement = null;
        selectionCallback = null;

        log('Element selection stopped');
    }

    function destroy() {
        stopSelection();
        if (messageHandler) {
            window.removeEventListener('message', messageHandler);
            messageHandler = null;
        }
        log('Element picker destroyed');
    }

    function getElementAtPoint(x, y) {
        return document.elementFromPoint(x, y);
    }

    function highlightElement(element) {
        if (!element) {
            hideHighlight();
            return;
        }
        updateHighlight(element);
    }

    function setHighlightColor(color) {
        CONFIG.highlightColor = color;
    }

    function setAllowedOrigin(origin) {
        CONFIG.allowedOrigin = origin || '*';
    }

    function setDebugMode(enabled) {
        CONFIG.debugMode = enabled;
    }

    function handleMessage(event) {
        var data = event.data || {};
        var type = data.type;
        var payload = data.payload;

        switch (type) {
            case 'START_SELECTION':
                startSelection(payload && payload.callback);
                break;

            case 'STOP_SELECTION':
                stopSelection();
                break;

            case 'HIGHLIGHT_ELEMENT':
                if (payload) {
                    if (payload.x !== undefined && payload.y !== undefined) {
                        var element = getElementAtPoint(payload.x, payload.y);
                        highlightElement(element);
                    } else if (payload.selector) {
                        try {
                            var foundElement = document.querySelector(payload.selector);
                            highlightElement(foundElement);
                        } catch (e) {
                            log('Invalid selector:', payload.selector);
                        }
                    }
                }
                break;

            case 'CLEAR_HIGHLIGHT':
                hideHighlight();
                break;

            case 'SET_HIGHLIGHT_COLOR':
                if (payload && payload.color) {
                    setHighlightColor(payload.color);
                }
                break;

            case 'SET_ALLOWED_ORIGIN':
                if (payload && payload.origin) {
                    setAllowedOrigin(payload.origin);
                }
                break;

            case 'GET_ELEMENT_AT_POINT':
                if (payload && payload.x !== undefined && payload.y !== undefined) {
                    var elem = getElementAtPoint(payload.x, payload.y);
                    var meta = getElementMetadata(elem);
                    try {
                        window.parent.postMessage({
                            type: 'ELEMENT_AT_POINT',
                            payload: {
                                metadata: meta,
                                x: payload.x,
                                y: payload.y,
                                timestamp: Date.now()
                            }
                        }, CONFIG.allowedOrigin);
                    } catch (err) {
                        console.error('[ElementPicker] Failed to send message:', err);
                    }
                }
                break;

            case 'QUERY_SELECTOR':
                if (payload && payload.selector) {
                    try {
                        var elems = document.querySelectorAll(payload.selector);
                        var metas = [];
                        for (var i = 0; i < elems.length && i < 50; i++) {
                            metas.push(getElementMetadata(elems[i]));
                        }
                        try {
                            window.parent.postMessage({
                                type: 'QUERY_SELECTOR_RESULT',
                                payload: {
                                    selector: payload.selector,
                                    count: elems.length,
                                    elements: metas,
                                    timestamp: Date.now()
                                }
                            }, CONFIG.allowedOrigin);
                        } catch (err) {
                            console.error('[ElementPicker] Failed to send message:', err);
                        }
                    } catch (e) {
                        try {
                            window.parent.postMessage({
                                type: 'QUERY_SELECTOR_ERROR',
                                payload: {
                                    selector: payload.selector,
                                    error: e.message,
                                    timestamp: Date.now()
                                }
                            }, CONFIG.allowedOrigin);
                        } catch (err) {
                            console.error('[ElementPicker] Failed to send message:', err);
                        }
                    }
                }
                break;

            default:
                break;
        }
    }

    messageHandler = handleMessage;
    window.addEventListener('message', messageHandler);

    window.VibeElementPicker = {
        startSelection: startSelection,
        stopSelection: stopSelection,
        destroy: destroy,
        getElementAtPoint: getElementAtPoint,
        highlightElement: highlightElement,
        setHighlightColor: setHighlightColor,
        setAllowedOrigin: setAllowedOrigin,
        setDebugMode: setDebugMode,
        generateCSSSelector: generateCSSSelector,
        generateXPath: generateXPath,
        getElementMetadata: getElementMetadata,
        isSelecting: function() { return isSelecting; },
        getConfig: function() { return CONFIG; }
    };

    log('VibeTerminal Element Picker initialized');

})();
